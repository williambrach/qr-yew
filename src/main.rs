use yew::prelude::*;
use yew::{function_component, html, use_state, Callback, Html};
extern crate qrcodegen;
use base64::{engine::general_purpose::STANDARD, Engine};
use qrcodegen::QrCode;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use web_sys::{Blob, HtmlElement, Url};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

// Encodes the provided text into a QR code SVG string with a specified error correction level.
pub fn text_2_qr_string(text: &str) -> String {
    // Set the error correction level to low (allows for faster QR code generation with less error correction).
    let errcorlvl: qrcodegen::QrCodeEcc = qrcodegen::QrCodeEcc::Low;
    // Encode the provided text into a QR code using the specified error correction level.
    let qr = QrCode::encode_text(text, errcorlvl).unwrap();
    // Convert the QR code into an SVG string representation with a border size of 4 units.
    let svg = to_svg_string_2(&qr, 4);
    // Return the generated SVG string.
    svg
}

// Converts the given QR code into an SVG path string with a specified border size.
fn to_svg_string_2(qr: &QrCode, border: i32) -> String {
    // Ensure the border is non-negative; if not, the function will panic.
    assert!(border >= 0, "Border must be non-negative");
    let mut result = String::new(); // Initialize an empty string to hold the SVG path data.
                                    // Iterate over each module (pixel) in the QR code.
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            // Check if the current module is black (true) or white (false).
            if qr.get_module(x, y) {
                // If not the first module, add a space to separate from the previous path command.
                if x != 0 || y != 0 {
                    result += " ";
                }
                // Add a path command for the current module. This draws a 1x1 square at the module's position,
                // offset by the border size.
                result += &format!("M{},{}h1v1h-1z", x + border, y + border);
            }
        }
    }
    // Return the complete SVG path data as a string.
    result
}

#[function_component(App)]
fn app() -> Html {
    let input_value = use_state(|| String::new());
    let qr_svg = use_state(|| String::new());
    let transparent_background = use_state(|| false);

    // Define state hooks for foreground and background colors
    let foreground_color = use_state(|| "#000000".to_string()); // Default: black
    let background_color = use_state(|| "#FFFFFF".to_string()); // Default: white

    // Callback for handling changes to the foreground color through a color picker input.
    // It updates the state holding the foreground color value.
    let on_foreground_color_change = {
        // Clone the `foreground_color` state handle to move it into the closure.
        let foreground_color = foreground_color.clone();
        Callback::from(move |e: Event| {
            // Safely cast the event target to an `HtmlInputElement` and update the foreground color state with its value.
            let input: HtmlInputElement = e.target_dyn_into().unwrap();
            foreground_color.set(input.value());
        })
    };

    // Callback for handling changes to the background color through a color picker input.
    // It updates the state holding the background color value.
    let on_background_color_change = {
        // Clone the `background_color` state handle to move it into the closure.
        let background_color = background_color.clone();
        Callback::from(move |e: Event| {
            // Safely cast the event target to an `HtmlInputElement` and update the background color state with its value.
            let input: HtmlInputElement = e.target_dyn_into().unwrap();
            background_color.set(input.value());
        })
    };

    // Callback for handling changes to the background transparency through a checkbox input.
    // It updates the state holding the background transparency value.
    let on_transparent_background_change = {
        // Clone the `transparent_background` state handle to move it into the closure.
        let transparent_background = transparent_background.clone();
        Callback::from(move |e: Event| {
            // Directly cast the event target to an `HtmlInputElement` without checking and update the transparency state with the checkbox's checked status.
            let input = e.target_unchecked_into::<HtmlInputElement>();
            transparent_background.set(input.checked());
        })
    };

    // Callback for handling generic input value changes.
    // It updates the state holding the current input value.
    let oninput = {
        // Clone the `input_value` state handle to move it into the closure.
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            // Directly cast the event target to an `HtmlInputElement` without checking and update the input value state with its value.
            let input: HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };

    // Callback for handling click events, typically for a button.
    // It generates a QR string based on the current input value and updates the state holding the QR code SVG.
    let onclick = {
        // Clone the necessary state handles to move them into the closure.
        let input_value = input_value.clone();
        let qr_svg = qr_svg.clone();
        Callback::from(move |_| {
            // Generate the QR code SVG string from the current input value and update the QR code state.
            let svg = text_2_qr_string(&input_value);
            qr_svg.set(svg);
        })
    };

    // Callback for downloading the QR code as an SVG file. This involves generating
    // the SVG content based on current state values (QR code data, foreground color,
    // background color, and transparency setting), creating a Blob from the SVG,
    // and then triggering a download.
    let download_svg = {
        // Clone state handles to move into the closure.
        let qr_svg = qr_svg.clone();
        let foreground_color = foreground_color.clone(); // Removed unnecessary underscore prefixes.
        let background_color = background_color.clone();
        let transparent_bg = transparent_background.clone();

        Callback::from(move |_| {
            // Access the global `window` object provided by the `web_sys` crate.
            if let Some(window) = web_sys::window() {
                // Generate the SVG content. The structure changes based on the transparency of the background.
                let svg_content = if *transparent_bg {
                    format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 29 29" stroke="none">
                       <path d="{}" fill="{}"/>
                       </svg>"#,
                        *qr_svg, *foreground_color
                    )
                } else {
                    format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 29 29" stroke="none">
                       <rect width="100%" height="100%" fill="{}"/>
                       <path d="{}" fill="{}"/>
                       </svg>"#,
                        *background_color, *qr_svg, *foreground_color
                    )
                };

                // Create a Blob from the SVG content string.
                let blob =
                    Blob::new_with_str_sequence(&js_sys::Array::of1(&svg_content.into())).unwrap();
                let url = Url::create_object_url_with_blob(&blob).unwrap();

                // Create a temporary anchor (`<a>`) element for triggering the download.
                let document = window.document().unwrap();
                let body = document.body().unwrap();
                let a = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap();

                // Set the `href` to the blob URL and specify the download file name.
                a.set_attribute("href", &url).unwrap();
                a.set_attribute("download", "qrcode.svg").unwrap();

                // Add the anchor to the document to make it clickable, then simulate a click.
                body.append_child(&a).unwrap();
                a.click();

                // Clean up by removing the anchor from the document and revoking the blob URL to release resources.
                body.remove_child(&a).unwrap();
                Url::revoke_object_url(&url).unwrap();
            }
        })
    };

    // Refactored function to download a QR code as a PNG image.
    // This function creates an SVG representation of the QR code, renders it onto a canvas,
    // converts the canvas content to a PNG, and then triggers a download of the PNG image.
    let download_png = {
        // Clone state variables to capture their current values inside the closure.
        let qr_svg = qr_svg.clone();
        let foreground_color = foreground_color.clone();
        let background_color = background_color.clone();
        let transparent_background = transparent_background.clone();

        // Create a Yew callback that will be triggered without an event argument.
        Callback::from(move |_| {
            // Ensure we have access to the window's document to manipulate the DOM.
            if let Some(window) = web_sys::window() {
                let document = window.document().unwrap();
                let body = document.body().unwrap();

                // Create an HTMLImageElement to load the SVG content.
                let img = document
                    .create_element("img")
                    .unwrap()
                    .dyn_into::<HtmlImageElement>()
                    .unwrap();

                // Create an HTMLCanvasElement to render the SVG for conversion to PNG.
                let canvas = document
                    .create_element("canvas")
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap();
                let ctx = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into::<CanvasRenderingContext2d>()
                    .unwrap();

                // Set the dimensions of the canvas.
                canvas.set_width(290); // Example dimension, adjust as needed.
                canvas.set_height(290);

                // Generate SVG content based on the current state (considering transparency).
                let svg_content = if *transparent_background {
                    format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 29 29">
                    <path d="{}" fill="{}"/></svg>"#,
                        *qr_svg, *foreground_color
                    )
                } else {
                    format!(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 29 29">
                    <rect width="100%" height="100%" fill="{}"/>
                    <path d="{}" fill="{}"/></svg>"#,
                        *background_color, *qr_svg, *foreground_color
                    )
                };

                // Convert SVG content to a base64-encoded data URL and set it as the source of the image.
                let svg_base64 = STANDARD.encode(svg_content);
                let img_src = format!("data:image/svg+xml;base64,{}", svg_base64);
                img.set_src(&img_src);

                // Clone the image element to use inside the onload closure.
                let img_clone = img.clone();

                // Define the onload event handler for the image. This renders the SVG onto the canvas
                // and then converts the canvas to a PNG data URL for downloading.
                let onload_closure = Closure::wrap(Box::new(move || {
                    ctx.draw_image_with_html_image_element(&img_clone, 0.0, 0.0)
                        .unwrap();
                    if let Ok(png_data_url) = canvas.to_data_url() {
                        // Create a temporary anchor element to initiate the download.
                        let a = document
                            .create_element("a")
                            .unwrap()
                            .dyn_into::<web_sys::HtmlElement>()
                            .unwrap();
                        a.set_attribute("href", &png_data_url).unwrap();
                        a.set_attribute("download", "qrcode.png").unwrap();
                        body.append_child(&a).unwrap();
                        a.click();
                        body.remove_child(&a).unwrap();
                    }
                }) as Box<dyn FnMut()>);

                // Set the onload event handler and ensure the closure is kept alive.
                img.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                onload_closure.forget(); // Prevents the closure from being garbage-collected.
            }
        })
    };

    html! {
        <>
        <style>
            { "
                body { font-family: 'Arial', sans-serif; background-color: #f0f0f0; margin: 40px; }
                h1 { color: #333; text-align: center; }
                .container { display: flex; flex-direction: column; align-items: center; gap: 20px; }
                input, button, .color-picker { padding: 10px; font-size: 16px; border-radius: 5px; border: 1px solid #ccc; }
                button { background-color: #007bff; color: white; cursor: pointer; }
                button:hover { background-color: #0056b3; }
                .qr-code { display: flex; justify-content: center; align-items: center; padding: 10px; background-color: white; width: 500px; height: 500px; }
                .color-picker-container { display: flex; gap: 10px; align-items: center;font-size: 14px;}
            " }
        </style>
        <div class="container">
            <h1>{ "QR Code Generator 🦀" }</h1>
            <input {oninput} value={(*input_value).clone()} style="width: 30%;" placeholder="Enter text to generate QR code"/>
            <div  class="color-picker-container">
                <p>{ "QR code color" }</p>
                <input type="color" onchange={on_foreground_color_change} class="color-picker" value={(*foreground_color).clone()}/>
                <div style={format!("width: 50px; height: 20px; background-color: {};", *foreground_color)}></div>
                <p>{" QR background color" }</p>
                <input type="color" onchange={on_background_color_change} class="color-picker" value={(*background_color).clone()}/>
                <div style={format!("width: 50px; height: 20px; background-color: {};", *background_color)}></div>
                <label for="transparent_background">{" Transparent Background"}</label>
                <input type="checkbox" id="transparent_background" onchange={on_transparent_background_change} checked={*transparent_background}/>
            </div>
            {
                if !input_value.is_empty() {
                    html! {
                        <>
                                <button {onclick}>{ "Generate QR Code" }</button>
                            {
                                if !qr_svg.is_empty() {
                                    html! {
                                        <div>
                                        <div class="qr-code">
                                        <svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="0 0 30 30" stroke="none">
                                        {
                                            if *transparent_background {
                                                html! {}
                                            } else {
                                                html! {<rect width="100%" height="100%" fill={(*background_color).clone()} />}
                                            }
                                        }
                                        <path d={(*qr_svg).clone()} fill={(*foreground_color).clone()}/>
                                        </svg>
                                        </div>
                                        <button onclick={download_svg.clone()}>{ "Download SVG" }</button>
                                        <button onclick={download_png.clone()}>{ "Download PNG" }</button>
                                    </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                        </>
                    }
                } else {
                    html! {}
                }
            }
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
