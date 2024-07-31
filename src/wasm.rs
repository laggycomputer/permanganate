use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn bingus() {
    alert("bingus");
}
