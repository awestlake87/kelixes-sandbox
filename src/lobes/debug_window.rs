
use std::mem;
use std::rc::Rc;
use std::thread;

use cortical;
use cortical::{ ResultExt };
use futures::{ Future, Sink };
use futures::sync::{ mpsc, oneshot };
use gdk_pixbuf::Pixbuf;
use gtk;
use gtk::{
    Window,
    WindowType,
    WidgetExt,
    ContainerExt,
    ImageExt,
    Image
};
use relm::{
    Relm,
    Widget,
    Update
};
use sc2;

use super::super::errors::{ Result, Error, ErrorKind };
use super::{ KeliConstraint, KeliData };

struct Model {
    close_receiver:         Option<oneshot::Receiver<()>>,

    pathing_receiver:       Option<mpsc::Receiver<sc2::data::ImageData>>,
    placement_receiver:     Option<mpsc::Receiver<sc2::data::ImageData>>,
    terrain_receiver:       Option<mpsc::Receiver<sc2::data::ImageData>>,

    creep_receiver:         Option<mpsc::Receiver<sc2::data::ImageData>>,
    visibility_receiver:    Option<mpsc::Receiver<sc2::data::ImageData>>,
}

#[derive(Msg)]
enum Msg {
    UpdatePathing(sc2::data::ImageData),
    UpdatePlacement(sc2::data::ImageData),
    UpdateTerrain(sc2::data::ImageData),

    UpdateCreep(sc2::data::ImageData),
    UpdateVisibility(sc2::data::ImageData),

    Quit
}

trait IntoPixbuf {
    fn into_pixbuf(self) -> Pixbuf;
}

impl IntoPixbuf for sc2::data::ImageData {
    fn into_pixbuf(self) -> Pixbuf {
        Pixbuf::new_from_vec(
            self.data,
            0,
            false,
            self.bits_per_pixel,
            self.width,
            self.height,
            self.width * 3
        )
    }
}

struct DebugWindow {
    _model: Model,

    pathing: Image,
    placement: Image,
    terrain: Image,

    creep: Image,
    visibility: Image,

    window: Window
}

impl Update for DebugWindow {
    type Model = Model;
    type ModelParam = (
        oneshot::Receiver<()>,

        mpsc::Receiver<sc2::data::ImageData>,
        mpsc::Receiver<sc2::data::ImageData>,
        mpsc::Receiver<sc2::data::ImageData>,

        mpsc::Receiver<sc2::data::ImageData>,
        mpsc::Receiver<sc2::data::ImageData>
    );
    type Msg = Msg;

    fn model(
        _: &Relm<Self>,
        params: (
            oneshot::Receiver<()>,

            mpsc::Receiver<sc2::data::ImageData>,
            mpsc::Receiver<sc2::data::ImageData>,
            mpsc::Receiver<sc2::data::ImageData>,

            mpsc::Receiver<sc2::data::ImageData>,
            mpsc::Receiver<sc2::data::ImageData>
        )
    )
        -> Model
    {
        Model {
            close_receiver: Some(params.0),

            pathing_receiver: Some(params.1),
            placement_receiver: Some(params.2),
            terrain_receiver: Some(params.3),

            creep_receiver: Some(params.4),
            visibility_receiver: Some(params.5)
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::UpdatePathing(pathing) => {
                self.pathing.set_from_pixbuf(Some(&pathing.into_pixbuf()))
            },
            Msg::UpdatePlacement(placement) => {
                self.placement.set_from_pixbuf(Some(&placement.into_pixbuf()))
            },
            Msg::UpdateTerrain(terrain) => {
                self.terrain.set_from_pixbuf(Some(&terrain.into_pixbuf()))
            },

            Msg::UpdateCreep(creep) => {
                self.creep.set_from_pixbuf(Some(&creep.into_pixbuf()))
            },
            Msg::UpdateVisibility(visibility) => {
                self.visibility.set_from_pixbuf(
                    Some(&visibility.into_pixbuf())
                )
            },

            Msg::Quit => gtk::main_quit()
        }
    }
}

impl Widget for DebugWindow {
    type Root = Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, mut model: Self::Model) -> Self {
        let window = Window::new(WindowType::Toplevel);
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let pathing = gtk::Image::new();
        let placement = gtk::Image::new();
        let terrain = gtk::Image::new();

        let creep = gtk::Image::new();
        let visibility = gtk::Image::new();

        relm.connect_exec_ignore_err(
            mem::replace(&mut model.close_receiver, None).unwrap(),
            |_| Msg::Quit
        );

        relm.connect_exec_ignore_err(
            mem::replace(&mut model.pathing_receiver, None).unwrap(),
            Msg::UpdatePathing
        );
        relm.connect_exec_ignore_err(
            mem::replace(&mut model.placement_receiver, None).unwrap(),
            Msg::UpdatePlacement
        );
        relm.connect_exec_ignore_err(
            mem::replace(&mut model.terrain_receiver, None).unwrap(),
            Msg::UpdateTerrain
        );

        relm.connect_exec_ignore_err(
            mem::replace(&mut model.creep_receiver, None).unwrap(),
            Msg::UpdateCreep
        );
        relm.connect_exec_ignore_err(
            mem::replace(&mut model.visibility_receiver, None).unwrap(),
            Msg::UpdateVisibility
        );

        vbox.add(&pathing);
        vbox.add(&placement);
        vbox.add(&terrain);
        vbox.add(&creep);
        vbox.add(&visibility);

        window.add(&vbox);

        window.show_all();

        Self {
            _model: model,
            pathing: pathing,
            placement: placement,
            terrain: terrain,
            creep: creep,
            visibility: visibility,
            window: window
        }
    }
}

pub struct DebugWindowLobe {
    last_updated:           u32,

    close_sender:           Option<oneshot::Sender<()>>,

    pathing_sender:         Option<mpsc::Sender<sc2::data::ImageData>>,
    placement_sender:       Option<mpsc::Sender<sc2::data::ImageData>>,
    terrain_sender:         Option<mpsc::Sender<sc2::data::ImageData>>,

    creep_sender:           Option<mpsc::Sender<sc2::data::ImageData>>,
    visibility_sender:      Option<mpsc::Sender<sc2::data::ImageData>>,

    window_thread:          Option<thread::JoinHandle<()>>,
}

impl DebugWindowLobe {
    pub fn new() -> Self {
        Self {
            last_updated: 0,
            close_sender: None,

            pathing_sender: None,
            placement_sender: None,
            terrain_sender: None,

            creep_sender: None,
            visibility_sender: None,

            window_thread: None
        }
    }

    fn start_window(&mut self) -> Result<()> {
        let (close_tx, close_rx) = oneshot::channel();

        let (pathing_tx, pathing_rx) = mpsc::channel(1);
        let (placement_tx, placement_rx) = mpsc::channel(1);
        let (terrain_tx, terrain_rx) = mpsc::channel(1);

        let (creep_tx, creep_rx) = mpsc::channel(1);
        let (visibility_tx, visibility_rx) = mpsc::channel(1);

        self.close_sender = Some(close_tx);

        self.pathing_sender = Some(pathing_tx);
        self.placement_sender = Some(placement_tx);
        self.terrain_sender = Some(terrain_tx);

        self.creep_sender = Some(creep_tx);
        self.visibility_sender = Some(visibility_tx);

        self.window_thread = Some(
            thread::spawn(move || {
                DebugWindow::run(
                    (
                        close_rx,

                        pathing_rx,
                        placement_rx,
                        terrain_rx,

                        creep_rx,
                        visibility_rx
                    )
                ).unwrap();
            })
        );

        Ok(())
    }

    fn close_window(&mut self) -> Result<()> {
        if let Some(sender) = mem::replace(&mut self.close_sender, None) {
            sender.send(()).map_err(
                |_| -> Error {
                    ErrorKind::Msg("unable to send close".into()).into()
                }
            )?;
        }

        Ok(())
    }

    fn join(&mut self) -> Result<()> {
        if let Some(hdl) = mem::replace(&mut self.window_thread, None) {
            hdl.join().map_err(
                |_| -> Error { ErrorKind::JoinError.into() }
            )?;
        }

        Ok(())
    }

    fn send_terrain_data(&mut self, frame: &sc2::FrameData) -> Result<()> {
        if let Some(sender) = mem::replace(&mut self.pathing_sender, None) {
            let mut pixels = sc2::data::ImageData {
                data: Vec::with_capacity(
                    frame.data.terrain_info.pathing_grid.data.len() * 3
                ),
                bits_per_pixel: 8,
                width: frame.data.terrain_info.pathing_grid.width,
                height: frame.data.terrain_info.pathing_grid.height
            };

            for p in &frame.data.terrain_info.pathing_grid.data {
                pixels.data.push(*p);
                pixels.data.push(*p);
                pixels.data.push(*p);
            }

            self.pathing_sender = Some(
                sender.send(pixels).wait().chain_err(
                    || cortical::ErrorKind::LobeError
                )?
            );
        }
        if let Some(sender) = mem::replace(&mut self.placement_sender, None) {
            let mut pixels = sc2::data::ImageData {
                data: Vec::with_capacity(
                    frame.data.terrain_info.placement_grid.data.len() * 3
                ),
                bits_per_pixel: 8,
                width: frame.data.terrain_info.placement_grid.width,
                height: frame.data.terrain_info.placement_grid.height
            };

            for p in &frame.data.terrain_info.placement_grid.data {
                pixels.data.push(*p);
                pixels.data.push(*p);
                pixels.data.push(*p);
            }

            self.placement_sender = Some(
                sender.send(pixels).wait().chain_err(
                    || cortical::ErrorKind::LobeError
                )?
            );
        }
        if let Some(sender) = mem::replace(&mut self.terrain_sender, None) {
            let mut pixels = sc2::data::ImageData {
                data: Vec::with_capacity(
                    frame.data.terrain_info.terrain_height.data.len() * 3
                ),
                bits_per_pixel: 8,
                width: frame.data.terrain_info.terrain_height.width,
                height: frame.data.terrain_info.terrain_height.height
            };

            for p in &frame.data.terrain_info.terrain_height.data {
                pixels.data.push(*p);
                pixels.data.push(*p);
                pixels.data.push(*p);
            }

            self.terrain_sender = Some(
                sender.send(pixels).wait().chain_err(
                    || cortical::ErrorKind::LobeError
                )?
            );
        }

        Ok(())
    }

    fn send_image_data(&mut self, frame: &sc2::FrameData) -> Result<()> {
        if let Some(sender) = mem::replace(&mut self.creep_sender, None) {
            let mut pixels = sc2::data::ImageData {
                data: Vec::with_capacity(frame.map.creep.data.len() * 3),
                bits_per_pixel: 8,
                width: frame.map.creep.width,
                height: frame.map.creep.height
            };

            for p in &frame.map.creep.data {
                let value = match *p {
                    0 => 0x00,
                    _ => 0xFF
                };

                pixels.data.push(value);
                pixels.data.push(value);
                pixels.data.push(value);
            }

            self.creep_sender = Some(
                sender.send(pixels).wait().chain_err(
                    || cortical::ErrorKind::LobeError
                )?
            );
        }

        if let Some(sender) = mem::replace(&mut self.visibility_sender, None) {
            let mut pixels = sc2::data::ImageData {
                data: Vec::with_capacity(
                    frame.map.visibility.data.len() * 3
                ),
                bits_per_pixel: 8,
                width: frame.map.visibility.width,
                height: frame.map.visibility.height
            };

            for p in &frame.map.visibility.data {
                let value = match *p {
                    0 => 0x30,
                    1 => 0x60,
                    2 => 0xFF,
                    _ => 0x00
                };

                pixels.data.push(value);
                pixels.data.push(value);
                pixels.data.push(value);
            }

            self.visibility_sender = Some(
                sender.send(pixels).wait().chain_err(
                    || cortical::ErrorKind::LobeError
                )?
            );
        }

        Ok(())
    }
}

impl Drop for DebugWindowLobe {
    fn drop(&mut self) {
        self.close_window().unwrap();
        self.join().unwrap();
    }
}

create_lobe_data! {
    module: debug_window,

    req frame: Rc<sc2::FrameData>,
}


pub use self::debug_window::{
    Input as DebugWindowInput,
    Output as DebugWindowOutput,
    FeedbackInput as DebugWindowFeedbackInput,
    FeedbackOutput as DebugWindowFeedbackOutput,
};

constrain_lobe! {
    lobe: DebugWindowLobe,
    constraint: KeliConstraint,
    data: KeliData,

    input: DebugWindowInput,
    output: DebugWindowOutput,
    feedback_input: DebugWindowFeedbackInput,
    feedback_output: DebugWindowFeedbackOutput,

    req frame: FrameData,
}

impl cortical::Lobe for DebugWindowLobe {
    type Input = DebugWindowInput;
    type Output = DebugWindowOutput;
    type FeedbackInput = DebugWindowFeedbackInput;
    type FeedbackOutput = DebugWindowFeedbackOutput;

    fn start(
        &mut self,
        _: cortical::NodeHdl,
        _: Vec<cortical::NodeHdl>,
        _: Vec<cortical::NodeHdl>
    )
        -> cortical::Result<()>
    {
        if !self.window_thread.is_none() {
            bail!("window already exists")
        }

        self.start_window().chain_err(|| cortical::ErrorKind::LobeError)?;

        Ok(())
    }
    fn update(&mut self, input: Self::Input) -> cortical::Result<()> {
        if self.last_updated == 0 {
            self.send_terrain_data(&input.frame).chain_err(
                || cortical::ErrorKind::LobeError
            )?;
            self.send_image_data(&input.frame).chain_err(
                || cortical::ErrorKind::LobeError
            )?;

            self.last_updated = input.frame.state.current_step;
        }
        if self.last_updated + 16 < input.frame.state.current_step {
            self.send_image_data(&input.frame).chain_err(
                || cortical::ErrorKind::LobeError
            )?;

            self.last_updated = input.frame.state.current_step;
        }

        Ok(())
    }
    fn tailor_output(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::Output>
    {
        Ok(DebugWindowOutput { })
    }

    fn tailor_feedback(&mut self, _: cortical::NodeHdl)
        -> cortical::Result<Self::FeedbackOutput>
    {
        Ok(DebugWindowFeedbackOutput { })
    }

    fn stop(&mut self) -> cortical::Result<()> {
        self.close_window().chain_err(|| cortical::ErrorKind::LobeError)?;
        self.join().chain_err(|| cortical::ErrorKind::LobeError)?;

        Ok(())
    }
}
