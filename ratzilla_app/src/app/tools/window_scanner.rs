// deep_extension_scanner.rs
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Document, Element, Node, HtmlElement};
use js_sys::{Object, Array, Reflect, Function};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use gloo::utils::format::JsValueSerdeExt;

// ============================================================================
// Window Object Deep Scanner
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowProperty {
    pub name: String,
    pub prop_type: String,
    pub value_preview: String,
    pub is_native: bool,
    pub is_function: bool,
    pub is_constructor: bool,
    pub enumerable: bool,
    pub configurable: bool,
    pub writable: Option<bool>,
    pub children_count: Option<usize>,
}

pub struct WindowScanner {
    baseline_properties: HashSet<String>,
    user_defined_globals: HashSet<String>,
}

impl WindowScanner {
    pub fn new() -> Self {
        Self {
            baseline_properties: Self::get_baseline_properties(),
            user_defined_globals: HashSet::new(),
        }
    }

    /// Define your own global variables to whitelist
    pub fn add_user_globals(&mut self, globals: Vec<String>) {
        for global in globals {
            self.user_defined_globals.insert(global);
        }
    }

    /// Get baseline browser properties (native APIs)
    fn get_baseline_properties() -> HashSet<String> {
        let mut baseline = HashSet::new();

        // Core JavaScript/Browser APIs
        let native_props = vec![
            // Constructors
            "Object", "Function", "Array", "String", "Number", "Boolean", "Symbol", "Date",
            "Promise", "RegExp", "Error", "Map", "Set", "WeakMap", "WeakSet", "Proxy",
            "Reflect", "JSON", "Math", "Intl", "ArrayBuffer", "SharedArrayBuffer",
            "DataView", "Float32Array", "Float64Array", "Int8Array", "Int16Array",
            "Int32Array", "Uint8Array", "Uint16Array", "Uint32Array", "Uint8ClampedArray",
            "BigInt", "BigInt64Array", "BigUint64Array",

            // Window properties
            "window", "self", "document", "name", "location", "history", "navigator",
            "screen", "alert", "confirm", "prompt", "console", "performance", "crypto",
            "indexedDB", "sessionStorage", "localStorage", "caches", "origin",

            // Timing functions
            "setTimeout", "clearTimeout", "setInterval", "clearInterval",
            "requestAnimationFrame", "cancelAnimationFrame", "requestIdleCallback",
            "cancelIdleCallback", "queueMicrotask",

            // Events
            "Event", "CustomEvent", "EventTarget", "AbortController", "AbortSignal",

            // DOM
            "Node", "Element", "HTMLElement", "Document", "DocumentFragment",
            "HTMLCollection", "NodeList", "DOMParser", "XMLSerializer",
            "MutationObserver", "IntersectionObserver", "ResizeObserver",
            "PerformanceObserver",

            // Fetch/Network
            "fetch", "Request", "Response", "Headers", "FormData", "URLSearchParams",
            "WebSocket", "XMLHttpRequest", "Blob", "File", "FileReader", "URL",

            // Web APIs
            "Worker", "SharedWorker", "ServiceWorker", "MessageChannel", "MessagePort",
            "BroadcastChannel", "ImageData", "ImageBitmap", "OffscreenCanvas",
            "WebAssembly", "WebGL2RenderingContext", "WebGLRenderingContext",
            "AudioContext", "MediaStream", "RTCPeerConnection",

            // CSS
            "CSS", "CSSStyleDeclaration", "StyleSheet", "MediaQueryList",

            // Other standard APIs
            "TextEncoder", "TextDecoder", "atob", "btoa", "isNaN", "isFinite",
            "parseInt", "parseFloat", "encodeURI", "decodeURI", "encodeURIComponent",
            "decodeURIComponent", "escape", "unescape", "eval",

            // Browser specific
            "chrome", "browser", "safari", // Browser objects (may not exist in all)
            "speechSynthesis", "Notification", "PaymentRequest",

            // Common browser extensions to baseline
            "__core-js_shared__", "__Zone_symbol__BLACK_LISTED_EVENTS",
            "webpackJsonp", // Common in many web apps
        ];

        for prop in native_props {
            baseline.insert(prop.to_string());
        }

        baseline
    }

    /// Deep scan all window properties
    pub fn scan_window_deep(&self) -> HashMap<String, Vec<WindowProperty>> {
        let window = window().unwrap();
        let mut results = HashMap::new();

        // Categorize findings
        results.insert("native".to_string(), Vec::new());
        results.insert("user_defined".to_string(), Vec::new());
        results.insert("potential_extensions".to_string(), Vec::new());
        results.insert("suspicious".to_string(), Vec::new());

        // Get all property names (including non-enumerable)
        let all_props = self.get_all_window_properties(&window);

        for prop_name in all_props {
            if let Some(prop_info) = self.analyze_property(&window, &prop_name) {
                // Categorize the property
                if self.baseline_properties.contains(&prop_name) {
                    results.get_mut("native").unwrap().push(prop_info);
                } else if self.user_defined_globals.contains(&prop_name) {
                    results.get_mut("user_defined").unwrap().push(prop_info);
                } else if self.is_likely_extension_property(&prop_name, &prop_info) {
                    results.get_mut("potential_extensions").unwrap().push(prop_info);
                } else {
                    results.get_mut("suspicious").unwrap().push(prop_info);
                }
            }
        }

        results
    }

    /// Get ALL properties including non-enumerable ones
    fn get_all_window_properties(&self, window: &web_sys::Window) -> Vec<String> {
        let mut all_props = HashSet::new();

        // Method 1: Object.keys (enumerable only)
        let keys = Object::keys(&window);
        for i in 0..keys.length() {
            if let Some(key) = keys.get(i).as_string() {
                all_props.insert(key);
            }
        }

        // Method 2: Object.getOwnPropertyNames (includes non-enumerable)
        let names = Object::get_own_property_names(&window);
        for i in 0..names.length() {
            if let Some(name) = names.get(i).as_string() {
                all_props.insert(name);
            }
        }

        // Method 3: for...in loop (includes prototype chain)
        let code = r#"
            const props = [];
            for (let prop in window) {
                props.push(prop);
            }
            return props;
        "#;

        if let Ok(result) = js_sys::eval(code) {
            if let Ok(array) = result.dyn_into::<Array>() {
                for i in 0..array.length() {
                    if let Some(prop) = array.get(i).as_string() {
                        all_props.insert(prop);
                    }
                }
            }
        }

        all_props.into_iter().collect()
    }

    /// Analyze a single property in detail
    fn analyze_property(&self, window: &web_sys::Window, prop_name: &str) -> Option<WindowProperty> {
        let prop_value = match Reflect::get(window, &JsValue::from_str(prop_name)) {
            Ok(val) => val,
            Err(_) => return None,
        };

        // Get property descriptor
        let descriptor = Object::get_own_property_descriptor(window, &JsValue::from_str(prop_name));

        // Extract descriptor properties
        let enumerable = Reflect::get(&descriptor, &JsValue::from_str("enumerable"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let configurable = Reflect::get(&descriptor, &JsValue::from_str("configurable"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let writable = Reflect::get(&descriptor, &JsValue::from_str("writable"))
            .ok()
            .and_then(|v| v.as_bool());

        // Determine type and other properties
        let prop_type = self.get_js_type(&prop_value);
        let is_function = prop_value.is_function();
        let is_constructor = self.is_constructor(&prop_value);
        let value_preview = self.get_value_preview(&prop_value, prop_name);
        let is_native = self.is_native_code(&prop_value);

        // Count children for objects
        let children_count = if prop_type == "object" && !prop_value.is_null() {
            if let Some(obj) = prop_value.dyn_ref::<Object>() {
                Some(Object::keys(obj).length() as usize)
            } else {
                None
            }
        } else {
            None
        };

        Some(WindowProperty {
            name: prop_name.to_string(),
            prop_type,
            value_preview,
            is_native,
            is_function,
            is_constructor,
            enumerable,
            configurable,
            writable,
            children_count,
        })
    }

    /// Get JavaScript type of value
    fn get_js_type(&self, value: &JsValue) -> String {
        if value.is_null() {
            "null".to_string()
        } else if value.is_undefined() {
            "undefined".to_string()
        } else if value.is_string() {
            "string".to_string()
        } else if value.is_function() {
            "function".to_string()
        } else if value.as_bool().is_some() {
            "boolean".to_string()
        } else if value.as_f64().is_some() {
            "number".to_string()
        } else if value.is_symbol() {
            "symbol".to_string()
        } else if value.is_bigint() {
            "bigint".to_string()
        } else {
            "object".to_string()
        }
    }

    /// Check if function is native code
    fn is_native_code(&self, value: &JsValue) -> bool {
        if !value.is_function() {
            return false;
        }

        if let Some(func) = value.dyn_ref::<Function>() {
            let func_str = func.to_string();
            func_str.includes("[native code]", 0)
        } else {
            false
        }
    }

    /// Check if function is a constructor
    fn is_constructor(&self, value: &JsValue) -> bool {
        if !value.is_function() {
            return false;
        }

        // Check if it has a prototype property
        Reflect::has(value, &JsValue::from_str("prototype")).unwrap_or(false)
    }

    /// Get a preview of the value
    fn get_value_preview(&self, value: &JsValue, prop_name: &str) -> String {
        if value.is_null() {
            "null".to_string()
        } else if value.is_undefined() {
            "undefined".to_string()
        } else if let Some(s) = value.as_string() {
            format!("\"{}\"", s.chars().take(50).collect::<String>())
        } else if let Some(n) = value.as_f64() {
            n.to_string()
        } else if let Some(b) = value.as_bool() {
            b.to_string()
        } else if value.is_function() {
            // Try to get function signature
            if let Some(func) = value.dyn_ref::<Function>() {
                return func.as_string().unwrap_or("???".to_string())
            } else {
                return "[Function]".to_string()
                }
            
        } else if value.is_symbol() {
            format!("Symbol({})", prop_name)
        } else {
            // For objects, try to get constructor name
            if let Ok(constructor) = Reflect::get(value, &JsValue::from_str("constructor")) {
                if let Ok(name) = Reflect::get(&constructor, &JsValue::from_str("name")) {
                    if let Some(name_str) = name.as_string() {
                        format!("[{} Object]", name_str)
                    } else {
                        "[Object]".to_string()
                    }
                } else {
                    "[Object]".to_string()
                }
            } else {
                "[Object]".to_string()
            }
        }
    }

    /// Heuristic to detect likely extension properties
    fn is_likely_extension_property(&self, name: &str, info: &WindowProperty) -> bool {
        // Common extension patterns
        let extension_patterns = vec![
            "__", // Double underscore prefix/suffix
            "DEVTOOLS",
            "Extension",
            "extension",
            "inject",
            "content",
            "background",
            "popup",
            "webpack", // Build tools
            "React", // Framework specific
            "Vue",
            "Angular",
            "$", // jQuery and similar
            "chrome",
            "browser",
        ];

        // Check name patterns
        for pattern in &extension_patterns {
            if name.contains(pattern) {
                return true;
            }
        }

        // Check if it's a non-native function/object added to window
        if !info.is_native && (info.is_function || info.prop_type == "object") {
            return true;
        }

        // Check for suspicious property descriptors
        if !info.enumerable && !info.is_native {
            return true;
        }

        false
    }
}

// ============================================================================
// DOM Element Scanner
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DOMElement {
    pub tag_name: String,
    pub id: Option<String>,
    pub class_list: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub is_custom_element: bool,
    pub is_shadow_host: bool,
    pub parent_info: Option<String>,
    pub children_count: usize,
    pub text_preview: Option<String>,
    pub inline_styles: Option<String>,
    pub data_attributes: HashMap<String, String>,
}

pub struct DOMScanner {
    whitelisted_ids: HashSet<String>,
    whitelisted_classes: HashSet<String>,
    whitelisted_tags: HashSet<String>,
}

impl DOMScanner {
    pub fn new() -> Self {
        Self {
            whitelisted_ids: HashSet::new(),
            whitelisted_classes: HashSet::new(),
            whitelisted_tags: HashSet::new(),
        }
    }

    /// Add your known element IDs
    pub fn whitelist_ids(&mut self, ids: Vec<String>) {
        for id in ids {
            self.whitelisted_ids.insert(id);
        }
    }

    /// Add your known CSS classes
    pub fn whitelist_classes(&mut self, classes: Vec<String>) {
        for class in classes {
            self.whitelisted_classes.insert(class);
        }
    }

    /// Add your known custom tags
    pub fn whitelist_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.whitelisted_tags.insert(tag);
        }
    }

    /// Scan entire DOM and categorize elements
    pub fn scan_dom_deep(&self) -> HashMap<String, Vec<DOMElement>> {
        let document = window().unwrap().document().unwrap();
        let mut results = HashMap::new();

        results.insert("whitelisted".to_string(), Vec::new());
        results.insert("suspicious".to_string(), Vec::new());
        results.insert("shadow_roots".to_string(), Vec::new());
        results.insert("custom_elements".to_string(), Vec::new());
        results.insert("hidden_elements".to_string(), Vec::new());

        // Start from document.documentElement (includes <html>)
        if let Some(root) = document.document_element() {
            self.scan_element_recursive(&root, &mut results);
        }

        // Also scan elements that might be outside normal flow
        self.scan_disconnected_elements(&document, &mut results);

        results
    }

    /// Recursively scan elements
    fn scan_element_recursive(&self, element: &Element, results: &mut HashMap<String, Vec<DOMElement>>) {
        let elem_info = self.analyze_element(element);

        // Categorize element
        if self.is_whitelisted(&elem_info) {
            results.get_mut("whitelisted").unwrap().push(elem_info.clone());
        } else if self.is_suspicious(&elem_info) {
            results.get_mut("suspicious").unwrap().push(elem_info.clone());
        }

        if elem_info.is_custom_element {
            results.get_mut("custom_elements").unwrap().push(elem_info.clone());
        }

        if elem_info.is_shadow_host {
            results.get_mut("shadow_roots").unwrap().push(elem_info.clone());
            // Try to access shadow root
            self.scan_shadow_root(element, results);
        }

        if self.is_hidden(element) {
            results.get_mut("hidden_elements").unwrap().push(elem_info.clone());
        }

        // Scan children using get_elements_by_tag_name("*")
        let children = element.get_elements_by_tag_name("*");
        for i in 0..children.length() {
            if let Some(child) = children.item(i) {
                self.scan_element_recursive(&child, results);
            }
        }
    }

    /// Analyze a single element
    fn analyze_element(&self, element: &Element) -> DOMElement {
        let tag_name = element.tag_name().to_lowercase();
        let id = element.id();
        let id = if id.is_empty() { None } else { Some(id) };

        // Get classes
        let class_name = element.class_name();
        let classes: Vec<String> = if class_name.is_empty() {
            Vec::new()
        } else {
            class_name.split_whitespace().map(|s| s.to_string()).collect()
        };

        // Get all attributes
        let mut attributes = HashMap::new();
        let mut data_attributes = HashMap::new();
        
        // Use get_attribute_names if available, otherwise use a known list
        let attr_names = vec!["style", "href", "src", "alt", "title", "type", "value", "name"];
        for name in attr_names {
            if let Some(value) = element.get_attribute(name) {
                attributes.insert(name.to_string(), value);
            }
        }
        
        // Check for common data attributes
        let data_attr_names = vec!["data-id", "data-type", "data-value", "data-extension"];
        for name in data_attr_names {
            if let Some(value) = element.get_attribute(name) {
                data_attributes.insert(name.to_string(), value);
            }
        }

        // Check for custom elements
        let is_custom_element = tag_name.contains('-');

        // Check for shadow root
        let is_shadow_host = self.has_shadow_root(element);

        // Get parent info
        let parent_info = element.parent_element().map(|parent| {
            let parent_id = parent.id();
            if parent_id.is_empty() {
                format!("<{}>", parent.tag_name().to_lowercase())
            } else {
                format!("<{} id=\"{}\">", parent.tag_name().to_lowercase(), parent_id)
            }
        });

        // Count children
        let children_count = element.child_element_count() as usize;

        // Get text preview (first 100 chars)
        let text_preview = element.text_content().map(|text| {
            let trimmed = text.trim();
            if trimmed.len() > 100 {
                format!("{}...", &trimmed[..100])
            } else {
                trimmed.to_string()
            }
        }).filter(|t| !t.is_empty());

        // Get inline styles
        let inline_styles = element.get_attribute("style");

        DOMElement {
            tag_name,
            id,
            class_list: classes,
            attributes,
            is_custom_element,
            is_shadow_host,
            parent_info,
            children_count,
            text_preview,
            inline_styles,
            data_attributes,
        }
    }

    /// Check if element has shadow root
    fn has_shadow_root(&self, element: &Element) -> bool {
        // Try to access shadowRoot property
        let result = js_sys::Reflect::get(element, &JsValue::from_str("shadowRoot"));
        if let Ok(shadow_root) = result {
            !shadow_root.is_null() && !shadow_root.is_undefined()
        } else {
            false
        }
    }

    /// Scan shadow DOM if accessible
    fn scan_shadow_root(&self, element: &Element, results: &mut HashMap<String, Vec<DOMElement>>) {
        if let Ok(shadow_root) = js_sys::Reflect::get(element, &JsValue::from_str("shadowRoot")) {
            if !shadow_root.is_null() && !shadow_root.is_undefined() {
                // Use eval to access shadow root children
                let code = format!(
                    r#"
                    (function() {{
                        const el = document.querySelector('[id="{}"]') || document.querySelector('{}');
                        if (el && el.shadowRoot) {{
                            return Array.from(el.shadowRoot.querySelectorAll('*'));
                        }}
                        return [];
                    }})()
                    "#,
                    element.id(),
                    element.tag_name()
                );
                
                if let Ok(result) = js_sys::eval(&code) {
                    if let Ok(array) = result.dyn_into::<Array>() {
                        for i in 0..array.length() {
                            if let Some(child) = array.get(i).dyn_ref::<Element>() {
                                self.scan_element_recursive(child, results);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check for disconnected elements (not in normal DOM tree)
    fn scan_disconnected_elements(&self, document: &Document, results: &mut HashMap<String, Vec<DOMElement>>) {
        // Look for elements that might be created but not attached
        let selectors = vec![
            "[style*='position: fixed']",
            "[style*='position: absolute']",
            "[style*='z-index: 9']",
            "[style*='z-index: 10']",
            "iframe",
            "object",
            "embed",
        ];

        for selector in selectors {
            if let Ok(node_list) = document.query_selector_all(selector) {
                for i in 0..node_list.length() {
                    if let Some(element) = node_list.item(i).and_then(|n| n.dyn_into::<Element>().ok()) {
                        let elem_info = self.analyze_element(&element);
                        if !self.is_whitelisted(&elem_info) {
                            results.get_mut("suspicious").unwrap().push(elem_info);
                        }
                    }
                }
            }
        }
    }

    /// Check if element is whitelisted
    fn is_whitelisted(&self, elem: &DOMElement) -> bool {
        // Check ID
        if let Some(id) = &elem.id {
            if self.whitelisted_ids.contains(id) {
                return true;
            }
        }

        // Check classes
        for class in &elem.class_list {
            if self.whitelisted_classes.contains(class) {
                return true;
            }
        }

        // Check tag
        if self.whitelisted_tags.contains(&elem.tag_name) {
            return true;
        }

        false
    }

    /// Check if element is suspicious
    fn is_suspicious(&self, elem: &DOMElement) -> bool {
        // Suspicious patterns
        let suspicious_patterns = vec![
            "extension", "inject", "content-script", "isolated",
            "__", "devtools", "grammarly", "lastpass", "adblock",
            "honey", "rakuten", "capital-one", "paypal", "ebates"
        ];

        // Check ID
        if let Some(id) = &elem.id {
            for pattern in &suspicious_patterns {
                if id.to_lowercase().contains(pattern) {
                    return true;
                }
            }
        }

        // Check classes
        for class in &elem.class_list {
            for pattern in &suspicious_patterns {
                if class.to_lowercase().contains(pattern) {
                    return true;
                }
            }
        }

        // Check data attributes
        for (key, _) in &elem.data_attributes {
            for pattern in &suspicious_patterns {
                if key.to_lowercase().contains(pattern) {
                    return true;
                }
            }
        }

        // Check for elements with no ID/class but complex inline styles
        if elem.id.is_none() && elem.class_list.is_empty() && elem.inline_styles.is_some() {
            if let Some(styles) = &elem.inline_styles {
                if styles.len() > 100 || styles.contains("!important") {
                    return true;
                }
            }
        }

        false
    }

    /// Check if element is hidden
    fn is_hidden(&self, element: &Element) -> bool {
        // Use JavaScript to check computed styles
        let code = format!(
            r#"
            (function() {{
                const el = document.querySelector('[id="{}"]') || document.querySelector('{}');
                if (el) {{
                    const style = window.getComputedStyle(el);
                    return style.display === 'none' || 
                           style.visibility === 'hidden' || 
                           style.opacity === '0';
                }}
                return false;
            }})()
            "#,
            element.id(),
            element.tag_name()
        );
        
        if let Ok(result) = js_sys::eval(&code) {
            result.as_bool().unwrap_or(false)
        } else {
            false
        }
    }
}

// ============================================================================
// Combined Scanner Results
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepScanResults {
    pub timestamp: f64,
    pub window_scan: HashMap<String, Vec<WindowProperty>>,
    pub dom_scan: HashMap<String, Vec<DOMElement>>,
    pub statistics: ScanStatistics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanStatistics {
    pub total_window_properties: usize,
    pub non_native_properties: usize,
    pub potential_extension_properties: usize,
    pub total_dom_elements: usize,
    pub suspicious_elements: usize,
    pub shadow_roots_found: usize,
    pub custom_elements_found: usize,
    pub hidden_elements_found: usize,
}

// ============================================================================
// Main Scanner Interface
// ============================================================================

#[wasm_bindgen]
pub struct DeepExtensionScanner {
    window_scanner: WindowScanner,
    dom_scanner: DOMScanner,
}

#[wasm_bindgen]
impl DeepExtensionScanner {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            window_scanner: WindowScanner::new(),
            dom_scanner: DOMScanner::new(),
        }
    }

    /// Configure whitelisted window globals
    #[wasm_bindgen(js_name = whitelistWindowGlobals)]
    pub fn whitelist_window_globals(&mut self, globals: Vec<JsValue>) {
        let globals: Vec<String> = globals.iter()
            .filter_map(|v| v.as_string())
            .collect();
        self.window_scanner.add_user_globals(globals);
    }

    /// Configure whitelisted DOM elements
    #[wasm_bindgen(js_name = whitelistDOMElements)]
    pub fn whitelist_dom_elements(&mut self, ids: Vec<JsValue>, classes: Vec<JsValue>, tags: Vec<JsValue>) {
        let ids: Vec<String> = ids.iter().filter_map(|v| v.as_string()).collect();
        let classes: Vec<String> = classes.iter().filter_map(|v| v.as_string()).collect();
        let tags: Vec<String> = tags.iter().filter_map(|v| v.as_string()).collect();

        self.dom_scanner.whitelist_ids(ids);
        self.dom_scanner.whitelist_classes(classes);
        self.dom_scanner.whitelist_tags(tags);
    }

    /// Run the complete deep scan
    #[wasm_bindgen(js_name = runDeepScan)]
    pub fn run_deep_scan(&self) -> JsValue {
        web_sys::console::log_1(&"Starting deep extension scan...".into());

        // Scan window object
        let window_results = self.window_scanner.scan_window_deep();

        // Scan DOM
        let dom_results = self.dom_scanner.scan_dom_deep();

        // Calculate statistics
        let stats = ScanStatistics {
            total_window_properties: window_results.values().map(|v| v.len()).sum(),
            non_native_properties: window_results.get("potential_extensions").map(|v| v.len()).unwrap_or(0)
                + window_results.get("suspicious").map(|v| v.len()).unwrap_or(0),
            potential_extension_properties: window_results.get("potential_extensions").map(|v| v.len()).unwrap_or(0),
            total_dom_elements: dom_results.values().map(|v| v.len()).sum(),
            suspicious_elements: dom_results.get("suspicious").map(|v| v.len()).unwrap_or(0),
            shadow_roots_found: dom_results.get("shadow_roots").map(|v| v.len()).unwrap_or(0),
            custom_elements_found: dom_results.get("custom_elements").map(|v| v.len()).unwrap_or(0),
            hidden_elements_found: dom_results.get("hidden_elements").map(|v| v.len()).unwrap_or(0),
        };

        let results = DeepScanResults {
            timestamp: js_sys::Date::now(),
            window_scan: window_results,
            dom_scan: dom_results,
            statistics: stats,
        };

        // Convert to JsValue using serde-wasm-bindgen
        JsValue::from_serde(&results).unwrap_or(JsValue::NULL)
    }

    /// Get a human-readable report
    #[wasm_bindgen(js_name = generateReport)]
    pub fn generate_report(&self) -> Vec<String> {
        let scan_value = self.run_deep_scan();
        if let Ok(results) = scan_value.into_serde::<DeepScanResults>() {
            self.format_report(&results)
        } else {
            vec!["Error generating report".to_string()]
        }
    }

    fn format_report(&self, results: &DeepScanResults) -> Vec<String> {
        let mut report = Vec::new();

        report.push(format!("=== Deep Extension Scan Report ===\n"));
        report.push(format!("Timestamp: {}\n\n", js_sys::Date::new(&JsValue::from(results.timestamp)).to_string()));

        // Statistics
        report.push("=== Statistics ===\n".to_string());
        report.push(format!("Total Window Properties: {}\n", results.statistics.total_window_properties));
        report.push(format!("Non-Native Properties: {}\n", results.statistics.non_native_properties));
        report.push(format!("Potential Extension Properties: {}\n", results.statistics.potential_extension_properties));
        report.push(format!("Total DOM Elements Scanned: {}\n", results.statistics.total_dom_elements));
        report.push(format!("Suspicious Elements: {}\n", results.statistics.suspicious_elements));
        report.push(format!("Shadow Roots Found: {}\n", results.statistics.shadow_roots_found));
        report.push(format!("Custom Elements: {}\n", results.statistics.custom_elements_found));
        report.push(format!("Hidden Elements: {}\n\n", results.statistics.hidden_elements_found));

        // Window scan results
        report.push("=== Window Object Analysis ===\n".to_string());

        if let Some(extensions) = results.window_scan.get("potential_extensions") {
            report.push("\nPotential Extension Properties:\n".to_string());
            for prop in extensions {
                report.push(format!("  - {} ({}): {}\n", 
                    prop.name, prop.prop_type, prop.value_preview));
                if !prop.is_native {
                    report.push(format!("    Non-native: true, Enumerable: {}, Configurable: {}\n",
                        prop.enumerable, prop.configurable));
                }
            }
        }

        if let Some(suspicious) = results.window_scan.get("suspicious") {
            report.push("\nSuspicious Window Properties:\n".to_string());
            for prop in suspicious {
                report.push(format!("  - {} ({}): {}\n", 
                    prop.name, prop.prop_type, prop.value_preview));
            }
        }

        // DOM scan results
        report.push("\n=== DOM Analysis ===\n".to_string());

        if let Some(suspicious) = results.dom_scan.get("suspicious") {
            report.push("\nSuspicious Elements:\n".to_string());
            for elem in suspicious {
                report.push(format!("  - <{}{}{}>\n",
                    elem.tag_name,
                    elem.id.as_ref().map(|id| format!(" id=\"{}\"", id)).unwrap_or_default(),
                    if elem.class_list.is_empty() { 
                        String::new() 
                    } else { 
                        format!(" class=\"{}\"", elem.class_list.join(" "))
                    }
                ));

                if !elem.data_attributes.is_empty() {
                    report.push("    Data attributes: ".to_string());
                    for (key, val) in &elem.data_attributes {
                        report.push(format!("{}=\"{}\" ", key, val));
                    }
                    report.push("\n".to_string());
                }

                if let Some(parent) = &elem.parent_info {
                    report.push(format!("    Parent: {}\n", parent));
                }
            }
        }

        if let Some(shadow_roots) = results.dom_scan.get("shadow_roots") {
            report.push("\nElements with Shadow DOM:\n".to_string());
            for elem in shadow_roots {
                report.push(format!("  - <{}{}>\n",
                    elem.tag_name,
                    elem.id.as_ref().map(|id| format!(" id=\"{}\"", id)).unwrap_or_default()
                ));
            }
        }

        report
    }
}

// ============================================================================
// Monitoring & Real-time Detection
// ============================================================================

#[wasm_bindgen]
pub struct ExtensionMonitor {
    window_baseline: HashSet<String>,
    mutation_observer: Option<web_sys::MutationObserver>,
    property_check_interval: Option<i32>,
}

#[wasm_bindgen]
impl ExtensionMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            window_baseline: Self::capture_window_baseline(),
            mutation_observer: None,
            property_check_interval: None,
        }
    }

    /// Start monitoring for changes
    #[wasm_bindgen(js_name = startMonitoring)]
    pub fn start_monitoring(&mut self, callback: &js_sys::Function) -> Result<(), JsValue> {
        // Start DOM monitoring
        self.start_dom_monitoring(callback)?;

        // Start window property monitoring
        self.start_window_monitoring(callback)?;

        Ok(())
    }

    /// Stop all monitoring
    #[wasm_bindgen(js_name = stopMonitoring)]
    pub fn stop_monitoring(&mut self) {
        // Stop DOM observer
        if let Some(observer) = &self.mutation_observer {
            observer.disconnect();
            self.mutation_observer = None;
        }

        // Stop interval
        if let Some(interval_id) = self.property_check_interval {
            window().unwrap().clear_interval_with_handle(interval_id);
            self.property_check_interval = None;
        }
    }

    fn capture_window_baseline() -> HashSet<String> {
        let window = window().unwrap();
        let mut baseline = HashSet::new();

        let names = Object::get_own_property_names(&window);
        for i in 0..names.length() {
            if let Some(name) = names.get(i).as_string() {
                baseline.insert(name);
            }
        }

        baseline
    }

    fn start_dom_monitoring(&mut self, callback: &js_sys::Function) -> Result<(), JsValue> {
        let callback_clone = callback.clone();

        let closure = Closure::wrap(Box::new(move |mutations: Array, _observer: JsValue| {
            for i in 0..mutations.length() {
                let mutation = mutations.get(i);
                let mutation: web_sys::MutationRecord = mutation.dyn_into().unwrap();

                // Check added nodes
                let added_nodes = mutation.added_nodes();
                for j in 0..added_nodes.length() {
                    if let Some(node) = added_nodes.get(j) {
                        if let Ok(element) = node.dyn_into::<Element>() {
                            let tag = element.tag_name();
                            let id = element.id();
                            let classes = element.class_name();

                            // Create change notification
                            let change = js_sys::Object::new();
                            let _ = Reflect::set(&change, &"type".into(), &"dom_addition".into());
                            
                            let elem_info = js_sys::Object::new();
                            let _ = Reflect::set(&elem_info, &"tag".into(), &tag.into());
                            let _ = Reflect::set(&elem_info, &"id".into(), &id.into());
                            let _ = Reflect::set(&elem_info, &"classes".into(), &classes.into());
                            let _ = Reflect::set(&elem_info, &"timestamp".into(), &js_sys::Date::now().into());
                            
                            let _ = Reflect::set(&change, &"element".into(), &elem_info);

                            // Call callback
                            let _ = callback_clone.call1(&JsValue::NULL, &change);
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(Array, JsValue)>);

        let observer = web_sys::MutationObserver::new(closure.as_ref().unchecked_ref())?;

        let mut options = web_sys::MutationObserverInit::new();
        options.set_child_list(true);
        options.set_subtree(true);
        options.set_attributes(true);

        let document = window().unwrap().document().unwrap();
        observer.observe_with_options(&document.body().unwrap(), &options)?;

        closure.forget();
        self.mutation_observer = Some(observer);

        Ok(())
    }

    fn start_window_monitoring(&mut self, callback: &js_sys::Function) -> Result<(), JsValue> {
        let baseline = self.window_baseline.clone();
        let callback_clone = callback.clone();

        let closure = Closure::wrap(Box::new(move || {
            let window = window().unwrap();
            let current_props = Self::capture_window_baseline();

            // Find new properties
            for prop in &current_props {
                if !baseline.contains(prop) {
                    // Analyze the new property
                    if let Ok(value) = Reflect::get(&window, &JsValue::from_str(prop)) {
                        let prop_type = if value.is_function() { "function" } 
                        else if value.is_object() { "object" }
                        else { "other" };

                        // Create change notification
                        let change = js_sys::Object::new();
                        let _ = Reflect::set(&change, &"type".into(), &"window_property_added".into());
                        
                        let prop_info = js_sys::Object::new();
                        let _ = Reflect::set(&prop_info, &"name".into(), &prop.as_str().into());
                        let _ = Reflect::set(&prop_info, &"type".into(), &prop_type.into());
                        let _ = Reflect::set(&prop_info, &"timestamp".into(), &js_sys::Date::now().into());
                        
                        let _ = Reflect::set(&change, &"property".into(), &prop_info);

                        // Call callback
                        let _ = callback_clone.call1(&JsValue::NULL, &change);
                    }
                }
            }
        }) as Box<dyn FnMut()>);

        // Check every 2 seconds
        let interval_id = window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                2000
            )?;

        closure.forget();
        self.property_check_interval = Some(interval_id);

        Ok(())
    }
}

// ============================================================================
// Required dependencies in Cargo.toml:
// ============================================================================
// [dependencies]
// wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
// wasm-bindgen-futures = "0.4"
// web-sys = { version = "0.3", features = [
//     "console",
//     "Document",
//     "Element",
//     "HtmlElement",
//     "Window",
//     "Node",
//     "NodeList",
//     "HtmlCollection",
//     "MutationObserver",
//     "MutationObserverInit",
//     "MutationRecord",
//     "Date",
// ]}
// js-sys = "0.3"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
