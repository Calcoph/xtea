use std::{time::Instant, collections::HashMap, ffi::OsString, path::{Path, PathBuf}};

use log::{error, trace};
use xtealib::ImguiLogger;
use imgui::{Context, Ui, sys::{ImGuiMemAllocFunc, ImGuiMemFreeFunc}};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use wgpu::{InstanceDescriptor, Device, Queue, Surface, CommandEncoder, SurfaceConfiguration};
use winit::{window::{self, Window}, dpi, event_loop::{EventLoop, ControlFlow}, event::{Event, WindowEvent}};
use image::io::Reader as ImageReader;
#[macro_use]
extern crate dlopen_derive;
use dlopen::wrapper::{Container, WrapperApi};

mod ui;

const ICON_PATH: &str = "icon.png";
const WIN_NAME: &str = "Xtea.png";

const SCREEN_HEIGHT: u32 = 200;
const SCREEN_WIDTH: u32 = 500;

static IMGUI_LOGGER: ImguiLogger = ImguiLogger::new();

#[derive(WrapperApi)]
struct PluginApi {
    init_logger: extern fn(logger: &'static ImguiLogger),
    init: extern fn(ctx: *mut imgui::sys::ImGuiContext, malloc: ImGuiMemAllocFunc, free: ImGuiMemFreeFunc),
    build_ui: extern fn(ui: &Ui),
    view_submenu: extern fn(ui: &Ui),
    get_name: extern fn() -> String
}

type Plugin = Container<PluginApi>;

struct Plugins {
    all_plugins: Vec<(String, PathBuf)>,
    loaded_plugins: HashMap<String, Plugin>,
    init_order: Vec<String>,
    ui_build_order: Vec<String>,
    view_submenu_order: Vec<String>
}

impl Plugins {
    fn new() -> Plugins {
        Plugins {
            all_plugins: Vec::new(),
            loaded_plugins: HashMap::new(),
            init_order: Vec::new(),
            ui_build_order: Vec::new(),
            view_submenu_order: Vec::new()
        }
    }

    fn load_all(&mut self) {
        let mut plugins_vec = Vec::new();

        let files = std::fs::read_dir("plugins").unwrap();
        for file in files {
            let file = file.unwrap();
            let filename = file.file_name();
            let extension = filename
                .to_str()
                .unwrap()
                .split(".")
                .last();
            if let Some("dll") = extension {
                let filename = Path::new("plugins").join(filename);
                let plugin = unsafe{ Container::<PluginApi>::load(&filename) };
                match plugin {
                    Ok(plugin) => plugins_vec.push((plugin, filename)),
                    Err(e) => {
                        error!("Failed to load plugin {}\n\t{}", file.file_name().to_str().unwrap(), e);
                    },
                }
            }
        };



        for (plugin, file_name) in plugins_vec {
            // TODO: load the orders from a file
            let name = plugin.get_name();
            self.all_plugins.push((name.clone(), file_name));
            self.loaded_plugins.insert(name.clone(), plugin);
            self.init_order.push(name.clone());
            self.ui_build_order.push(name.clone());
            self.view_submenu_order.push(name);
        };

        for plugin in self.init_iter() {
            init_plugin(plugin);
        }
    }

    fn init_iter(&self) -> impl Iterator<Item=&Plugin> {
        self.init_order.iter().filter_map(|name| self.loaded_plugins.get(name))
    }

    fn ui_build_iter(&self) -> impl Iterator<Item=&Plugin> {
        self.ui_build_order.iter().filter_map(|name| self.loaded_plugins.get(name))
    }

    fn view_submenu_iter(&self) -> impl Iterator<Item=&Plugin> {
        self.view_submenu_order.iter().filter_map(|name| self.loaded_plugins.get(name))
    }

    fn reload_all_plugins(&mut self) {
        // To avoid cloning the self.all_plugins
        let mut tmp_vec = vec![];
        std::mem::swap(&mut self.all_plugins, &mut tmp_vec);

        for (plugin_name, dll) in tmp_vec.iter() {
            self.reload_plugin_dll(plugin_name, dll);
        }

        std::mem::swap(&mut self.all_plugins, &mut tmp_vec);
    }

    fn reload_plugin(&mut self, name: &str) {
        let (name, dll_path) = self.all_plugins.iter().find(|(plugin_name, _)| name == plugin_name).unwrap().clone();

        self.reload_plugin_dll(&name, &dll_path)
    }

    fn reload_plugin_dll(&mut self, name: &str, dll_path: &PathBuf) {
        if let Some(plugin) = self.loaded_plugins.remove(name) {
            std::mem::drop(plugin);
            let plugin = unsafe{ Container::<PluginApi>::load(dll_path) };
            match plugin {
                Ok(plugin) => {
                    init_plugin(&plugin);
                    self.loaded_plugins.insert(name.to_string(), plugin);
                },
                Err(e) => {
                    error!("Failed to reload plugin {} ({})\n\t{}", dll_path.to_str().unwrap(), name, e);
                },
            }
        }
    }

    fn unload(&mut self, plugin_name: &str) {
        self.loaded_plugins.remove(plugin_name);
    }

    fn activate(&mut self, dll_path: &PathBuf) {
        let plugin = unsafe{ Container::<PluginApi>::load(dll_path) };
            match plugin {
                Ok(plugin) => {
                    init_plugin(&plugin);
                    self.loaded_plugins.insert(plugin.get_name(), plugin);
                },
                Err(e) => {
                    error!("Failed to load plugin {} \n\t{}", dll_path.to_str().unwrap(), e);
                },
            }
    }
}

fn init_plugin(plugin: &Plugin) {
    let ctx = unsafe {imgui::sys::igGetCurrentContext()};
    let malloc = &mut None;
    let free = &mut None;
    let user_data = &mut std::ptr::null_mut();
    unsafe {imgui::sys::igGetAllocatorFunctions(malloc, free, user_data)};
    plugin.init_logger(&IMGUI_LOGGER);
    plugin.init(ctx, *malloc, *free);
}

fn main() {
    if true {
        IMGUI_LOGGER.init().unwrap();
    }
    let event_loop = EventLoop::new();
    let window = make_window(&event_loop);

    let mut context = Context::create();
    let mut platform = imgui_winit_support::WinitPlatform::init(&mut context);
    platform.attach_window(context.io_mut(), &window, imgui_winit_support::HiDpiMode::Default);
    
    let (device, queue, config, surface) = pollster::block_on(init_gpu(&window));

    let renderer_config = RendererConfig {
        texture_format: config.format,
        ..Default::default()
    };

    let renderer = Renderer::new(&mut context, &device, &queue, renderer_config);

    let mut plugins = Plugins::new();
    plugins.load_all();
    let mut state = State::new(window, platform, context, surface, device, renderer, queue, config, plugins);

    event_loop.run(move |event,_window_target,control_flow| {
        state.run_event_loop(event, control_flow)
    });
}

async fn init_gpu(window: &Window) -> (Device, Queue, SurfaceConfiguration, Surface) {
    let backends = wgpu::Backends::all().difference(wgpu::Backends::DX12);
    let instance = wgpu::Instance::new(InstanceDescriptor {
        backends,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
    });

    let surface = unsafe { instance.create_surface(window).unwrap() };

    let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), //TODO: add option to select between LowPower, HighPerformance or default
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS, // TODO: Make this optional
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits {
                        max_push_constant_size: 4,
                        ..wgpu::Limits::downlevel_webgl2_defaults()
                    }
                } else {
                    wgpu::Limits {
                        max_push_constant_size: 4,
                        ..wgpu::Limits::default()
                    }
                },
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![surface.get_capabilities(&adapter).formats[0]],
    };
    surface.configure(&device, &config);

    (device, queue, config, surface)
}

fn make_window(event_loop: &EventLoop<()>) -> Window {
    let icon = match ImageReader::open(ICON_PATH).unwrap().decode() {
        Ok(img) => Ok(img.to_rgba8()),
        Err(_) => Err(InitError::Unkown),
    }.unwrap();

    let wb = window::WindowBuilder::new()
        .with_title(WIN_NAME)
        .with_inner_size(dpi::LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
        .with_window_icon(Some(match window::Icon::from_rgba(icon.into_raw(), 64, 64) {
            Ok(icon) => icon,
            Err(_) => panic!("Couldn't get icon raw data")
        }));

    wb.build(&event_loop)
        .unwrap()
}

#[derive(Debug)]
enum InitError {
    Unkown
}

struct State {
    window: Window,
    platform: WinitPlatform,
    context: Context,
    surface: Surface,
    device: Device,
    renderer: Renderer,
    queue: Queue,
    config: SurfaceConfiguration,
    ui_state: ui::UiState,
    plugins: Plugins,
    last_render_time: Instant,
}

impl State {
    fn new(window: Window, platform: WinitPlatform, mut context: Context, surface: Surface, device: Device, renderer: Renderer, queue: Queue, config: SurfaceConfiguration, plugins: Plugins) -> State {
        context.io_mut().config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;
        State {
            window,
            platform,
            context,
            surface,
            device,
            renderer,
            queue,
            config,
            plugins,
            ui_state: ui::UiState::new(),
            last_render_time: Instant::now()
        }
    }

    fn run_event_loop<'a>(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        self.platform.handle_event(self.context.io_mut(), &self.window, &event);
        match event {
            Event::WindowEvent { window_id, event } if window_id == self.window.id() => {
                match event {
                    WindowEvent::Resized(size) => {
                        self.resize(size)
                    },
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit, //control_flow is a pointer to the next action we wanna do. In this case, exit the program
                    WindowEvent::ScaleFactorChanged { scale_factor: _, new_inner_size } => {
                        self.resize(*new_inner_size)
                    },
                    _ => ()
                }
            },
            Event::Suspended => *control_flow = ControlFlow::Wait,
            Event::MainEventsCleared => {
                self.window.request_redraw()
            },
            Event::RedrawRequested(window_id) => if window_id == self.window.id() {
                let now = std::time::Instant::now();
                //let dt = now - self.last_render_time;
                self.last_render_time = now;
    
                //state.borrow_mut().update(dt, &gpu.borrow());
                let output = self.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                //let mut encoders = te_renderer::state::TeState::prepare_render(&gpu.borrow());
    
                let imgui_encoder = self.render(&view);
                self.queue.submit(vec![imgui_encoder.finish()]);
                output.present();
            },
            _ => ()
        }
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
    ) -> CommandEncoder {
        self.platform.prepare_frame(self.context.io_mut(), &self.window).expect("Failed to prepare frame");
        let ui = self.context.frame();
    
        ui::create_ui(&ui, &mut self.ui_state, &mut self.plugins);
    
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ImGui Render Encoder")
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.renderer.render(self.context.render(), &self.queue, &self.device, &mut render_pass).expect("Rendering failed");
        }
        encoder
    }

    fn resize(&mut self, new_size: dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }
}
