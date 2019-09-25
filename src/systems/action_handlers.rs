use crate::common::{cwd_path, Action, Error, Scene};
use crate::components::*;
use crate::resources::State;
use libflate::gzip::{Decoder, Encoder};
use specs::{Entities, Entity, Join, LazyUpdate, Read, ReadStorage, System, Write, WriteStorage};
use std::path::{Path, PathBuf};

pub struct ActionHandler;

impl ActionHandler {
    fn deselect(&mut self, e: &Entities, s: &ReadStorage<Selection>, u: &LazyUpdate) {
        for (entity, _) in (e, s).join() {
            u.remove::<Selection>(entity);
        }
    }

    fn select_next(
        &mut self,
        e: &Entities,
        sel: &ReadStorage<Selectable>,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
        keep: bool,
    ) {
        let mut all: Vec<(Entity, bool)> = Vec::default();
        let mut start = 0usize;

        for (i, (entity, _)) in (e, sel).join().enumerate() {
            let is_selected = s.get(entity).is_some();
            all.push((entity, is_selected));
            if is_selected {
                start = i
            }
        }

        // go through unselected items only, starting with last selection known
        let mut unselected_iter = all
            .iter()
            .cycle()
            .skip(start)
            .take(all.len())
            .filter(|(_, is_sel)| !is_sel);

        if !keep {
            self.deselect(e, s, u);
        }

        if let Some(entity) = unselected_iter.next() {
            u.insert(entity.0, Selection); // select next if possible
        } else if let Some(entity) = all.first() {
            u.insert(entity.0, Selection); // select first if "redeselecting"
        }
    }

    fn delete_selected(&mut self, e: &Entities, s: &ReadStorage<Selection>) -> Option<Error> {
        let mut deleted = 0usize;

        for (entity, _) in (e, s).join() {
            if let Err(_) = e.delete(entity) {
                return Some(Error::ExecutionError(String::from("Error deleting entity")));
            } else {
                deleted += 1;
            }
        }

        if deleted == 0 {
            return Some(Error::ExecutionError(String::from("No entity to delete")));
        }

        None
    }

    fn translate_selected(
        &mut self,
        t: Translation,
        p: &mut WriteStorage<Position>,
        s: &ReadStorage<Selection>,
        d: &ReadStorage<Dimension>,
    ) {
        for (position, _, dimension) in (p, s, d).join() {
            position.apply(t, dimension.w, dimension.h);
        }
    }

    fn import_sprite(
        &mut self,
        e: &Entities,
        sprite: Sprite,
        s: &ReadStorage<Selection>,
        u: &LazyUpdate,
        pos: Option<Position>,
    ) -> Result<(), Error> {
        self.deselect(e, s, u);
        let entity = e.create();

        u.insert(entity, Dimension::for_sprite(&sprite)?);
        u.insert(entity, pos.unwrap_or(Position::from_xyz(10, 10, 0)));
        u.insert(entity, Color::default());
        u.insert(entity, Selection);
        u.insert(entity, Selectable);
        u.insert(entity, Border);
        u.insert(entity, sprite);

        Ok(())
    }

    fn save_scene(
        &mut self,
        s: &ReadStorage<Sprite>,
        p: &WriteStorage<Position>,
        path: &str,
    ) -> Result<(), Error> {
        let ronified = ron::ser::to_string(&Scene::from((s, p)))?;
        let raw_path = if Path::new(path).extension() != Some(std::ffi::OsStr::new("rgz")) {
            Path::new(path).with_extension("rgz")
        } else {
            PathBuf::from(path)
        };
        let abs_path = cwd_path(&raw_path)?;

        let file = std::fs::File::create(abs_path)?;
        let mut encoder = Encoder::new(file)?;

        use std::io::Write;
        encoder.write(ronified.as_ref())?;
        encoder.finish().into_result()?;
        Ok(())
    }

    fn load_scene(
        &mut self,
        e: &Entities,
        s: &ReadStorage<Selection>,
        sp: &ReadStorage<Sprite>,
        u: &LazyUpdate,
        path: &str,
    ) -> Result<(), Error> {
        let abs_path = cwd_path(Path::new(path))?;
        let file = std::fs::File::open(abs_path)?;

        let decoder = Decoder::new(file)?;
        let scene: Scene = ron::de::from_reader(decoder)?;

        self.apply_scene(scene, e, s, sp, u)
    }

    fn clear_scene(&mut self, e: &Entities, sp: &ReadStorage<Sprite>) -> Result<(), Error> {
        for (entity, _) in (e, sp).join() {
            e.delete(entity)?;
        }

        Ok(())
    }

    fn apply_scene(
        &mut self,
        scene: Scene,
        e: &Entities,
        s: &ReadStorage<Selection>,
        sp: &ReadStorage<Sprite>,
        u: &LazyUpdate,
    ) -> Result<(), Error> {
        self.clear_scene(e, sp)?;

        for obj in scene.objects {
            self.import_sprite(e, obj.0, s, u, Some(obj.1))?;
        }

        self.deselect(e, s, u);

        Ok(())
    }
}

impl<'a> System<'a> for ActionHandler {
    type SystemData = (
        Entities<'a>,
        Write<'a, State>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Selectable>,
        ReadStorage<'a, Selection>,
        ReadStorage<'a, Dimension>,
        ReadStorage<'a, Sprite>,
        Read<'a, LazyUpdate>,
    );

    fn run(&mut self, (e, mut state, mut p, sel, s, d, sp, u): Self::SystemData) {
        while let Some(action) = state.pop_action() {
            match action {
                Action::None => {}
                Action::ClearError => state.set_error(None),
                Action::SetMode(mode) => state.set_mode(mode),
                Action::ReverseMode => state.reverse_mode(),
                Action::Deselect => self.deselect(&e, &s, &u),
                Action::SelectNext(keep) => self.select_next(&e, &sel, &s, &u, keep),
                Action::Translate(t) => self.translate_selected(t, &mut p, &s, &d),
                Action::Delete => state.set_error(self.delete_selected(&e, &s)),
                Action::Import(sprite) => {
                    if let Err(err) = self.import_sprite(&e, sprite, &s, &u, None) {
                        state.set_error(Some(err));
                    }
                }
                Action::Save(path) => {
                    if let Err(err) = self.save_scene(&sp, &p, &path) {
                        state.set_error(Some(err));
                    }
                }
                Action::Load(path) => {
                    if let Err(err) = self.load_scene(&e, &s, &sp, &u, &path) {
                        state.set_error(Some(err));
                    }
                }
            }
        }
    }
}
