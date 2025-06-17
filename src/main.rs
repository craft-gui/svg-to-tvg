use craft::events::ui_events::pointer::PointerButtonUpdate;
use craft::style::{AlignItems, Overflow};
use craft::WindowContext;
use craft::{
    components::{Component, ComponentId, ComponentSpecification, Event},
    elements::{Container, ElementStyles, Text},
    style::{Display, FlexDirection},
    Color,
};
use rfd::AsyncFileDialog;
use tinyvg_rs::svg_to_tvg::svg_to_tvg::svg_to_tvg;

#[derive(Default)]
pub struct Application {
}

#[derive(Default)]
pub struct GlobalState {
    converted_files: Vec<File>
}

#[derive(Clone)]
#[derive(Debug)]
pub struct File {
    name: String,
    data: Vec<u8>,
}

pub enum AppMessage {
    ConvertedFiles(Vec<File>)
}

#[derive(Default)]
pub struct UploadFileButton {
}

impl UploadFileButton {
    fn upload_files(event: &mut Event) {
        let future = async move {
            let mut files: Vec<File> = vec![];
            if let Some(file_handles) = AsyncFileDialog::new().pick_files().await {
                for file_handle in file_handles {
                    let mut file_name = file_handle.file_name();
                    file_name = file_name.replace(".svg", "");
                    file_name.push_str(".tvg");
                    
                    let data = file_handle.read().await;
                    let tvg_data = svg_to_tvg(&data);

                    let file = File {
                        name: file_name,
                        data: tvg_data,
                    };
                    files.push(file);
                }
            }

            Event::async_result(AppMessage::ConvertedFiles(files))  
        };
        
        event.future(future);
    }
}

impl Component for UploadFileButton {
    type GlobalState = GlobalState;
    type Props = ();
    type Message = AppMessage;

    fn view(&self, _global_state: &Self::GlobalState, _props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window_context: &WindowContext) -> ComponentSpecification {
        Container::new().push(Text::new("Upload")
            .font_size(16)
            .padding("10px", "15px", "10px", "15px")
            .border_radius(4.0, 4.0, 4.0, 4.0)
            .background(Color::from_rgb8(31, 132, 253))
            .color(Color::WHITE)
            .on_pointer_button_up(
                move |_state: &mut Self, _global_state: &mut Self::GlobalState, event: &mut Event, _pointer_button: &PointerButtonUpdate| {
                    Self::upload_files(event);
                },
            ))
            .component()
    }

    fn on_user_message(
        &mut self,
        global_state: &mut Self::GlobalState,
        _props: &Self::Props,
        _event: &mut Event,
        message: &Self::Message,
    ) {
        let AppMessage::ConvertedFiles(uploaded_files) = message;
        global_state.converted_files.extend(uploaded_files.iter().cloned());
    }
}


impl Application {
    fn download_file(event: &mut Event, file_name: &String, file_data: &[u8]) {
        event.prevent_propagate();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::{window, Blob, BlobPropertyBag, Url};
            use js_sys::Uint8Array;

            let array = Uint8Array::from(file_data);
            let mut blob_parts = js_sys::Array::new();
            blob_parts.push(&array);

            let blob = Blob::new_with_u8_array_sequence_and_options(
                &blob_parts,
                BlobPropertyBag::new().type_("application/octet-stream"),
            ).unwrap();

            let url = Url::create_object_url_with_blob(&blob).unwrap();
            let document = window().unwrap().document().unwrap();

            let a = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
            a.set_href(&url);
            a.set_download(&file_name);
            a.style().set_property("display", "none").unwrap();

            document.body().unwrap().append_child(&a).unwrap();
            a.click();
            document.body().unwrap().remove_child(&a).unwrap();
            Url::revoke_object_url(&url).unwrap();
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::fs;
            use rfd::FileDialog;

            if let Some(path) = FileDialog::new()
                .set_file_name(file_name)
                .save_file()
            {
                let _ = fs::write(path, &file_data);
            }
        }
    }
}

impl Component for Application {
    type GlobalState = GlobalState;
    type Props = ();
    type Message = AppMessage;

    fn view(&self, global_state: &Self::GlobalState, _props: &Self::Props, _children: Vec<ComponentSpecification>, _id: ComponentId, _window_context: &WindowContext) -> ComponentSpecification {
        let wrapper = Container::new().background(Color::WHITE).width("100%").height("100%").overflow_y(Overflow::Scroll);

        let mut uploaded_files = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .gap("20px")
            .push(Text::new("Uploaded Files").font_size(20.0).margin("20px", "0px", "0px", "0px"))
            ;

        for file in &global_state.converted_files {

            let row = Container::new()
                .display(Display::Flex)
                .align_items(AlignItems::Center)
                .gap("20px")
                .push(Text::new(file.name.as_str()))
                .push(
                    Text::new("Download")
                        .margin("0px", "0px", "0px", "auto")
                        .font_size(16)
                        .padding("10px", "15px", "10px", "15px")
                        .border_radius(4.0, 4.0, 4.0, 4.0)
                        .background(Color::from_rgb8(34, 197, 94))
                        .color(Color::WHITE)
                        .on_pointer_button_up({
                            let file_data = file.data.clone();
                            let file_name = file.name.clone();

                            move |_state: &mut Self, _global_state: &mut Self::GlobalState, event: &mut Event, _pointer_button: &PointerButtonUpdate| {
                                Self::download_file(event, &file_name, file_data.as_ref());
                            }
                        })

                );
            uploaded_files.push_in_place(row.component());
        }

        let app = Container::new()
            .display(Display::Flex)
            .flex_direction(FlexDirection::Column)
            .max_width("1440px")
            .width("100%")
            .height("100%")
            .padding("20px", "20px", "20px", "20px")
            .gap("10px")
            .push(Text::new("SVG to TVG Converter").font_size(32.0))
            .push(Container::new().push(Text::new("")))
            .push(UploadFileButton::component())
            .push(uploaded_files)
            .component();

        wrapper.push(app).component()
    }
}

#[allow(unused)]
#[cfg(not(target_os = "android"))]
fn main() {
    use craft::CraftOptions;
    craft::craft_main(Application::component(), GlobalState::default(), CraftOptions::basic("SVG to TVG Converter"));
}
