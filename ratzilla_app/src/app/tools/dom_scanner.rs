
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
                    if let Some(element) = node_list.item(i).and_then(|n| n.dyn_ref::<Element>()) {
                        let elem_info = self.analyze_element(element);
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
    pub fn generate_report(&self) -> String {
        let scan_value = self.run_deep_scan();
        if let Ok(results) = scan_value.into_serde::<DeepScanResults>() {
            self.format_report(&results)
        } else {
            "Error generating report".to_string()
        }
    }

    fn format_report(&self, results: &DeepScanResults) -> String {
        let mut report = String::new();

        report.push_str(&format!("=== Deep Extension Scan Report ===\n"));
        report.push_str(&format!("Timestamp: {}\n\n", js_sys::Date::new(&JsValue::from(results.timestamp)).to_string()));

        // Statistics
        report.push_str("=== Statistics ===\n");
        report.push_str(&format!("Total Window Properties: {}\n", results.statistics.total_window_properties));
        report.push_str(&format!("Non-Native Properties: {}\n", results.statistics.non_native_properties));
        report.push_str(&format!("Potential Extension Properties: {}\n", results.statistics.potential_extension_properties));
        report.push_str(&format!("Total DOM Elements Scanned: {}\n", results.statistics.total_dom_elements));
        report.push_str(&format!("Suspicious Elements: {}\n", results.statistics.suspicious_elements));
        report.push_str(&format!("Shadow Roots Found: {}\n", results.statistics.shadow_roots_found));
        report.push_str(&format!("Custom Elements: {}\n", results.statistics.custom_elements_found));
        report.push_str(&format!("Hidden Elements: {}\n\n", results.statistics.hidden_elements_found));

        // Window scan results
        report.push_str("=== Window Object Analysis ===\n");

        if let Some(extensions) = results.window_scan.get("potential_extensions") {
            report.push_str("\nPotential Extension Properties:\n");
            for prop in extensions {
                report.push_str(&format!("  - {} ({}): {}\n", 
                    prop.name, prop.prop_type, prop.value_preview));
                if !prop.is_native {
                    report.push_str(&format!("    Non-native: true, Enumerable: {}, Configurable: {}\n",
                        prop.enumerable, prop.configurable));
                }
            }
        }

        if let Some(suspicious) = results.window_scan.get("suspicious") {
            report.push_str("\nSuspicious Window Properties:\n");
            for prop in suspicious {
                report.push_str(&format!("  - {} ({}): {}\n", 
                    prop.name, prop.prop_type, prop.value_preview));
            }
        }

        // DOM scan results
        report.push_str("\n=== DOM Analysis ===\n");

        if let Some(suspicious) = results.dom_scan.get("suspicious") {
            report.push_str("\nSuspicious Elements:\n");
            for elem in suspicious {
                report.push_str(&format!("  - <{}{}{}>\n",
                    elem.tag_name,
                    elem.id.as_ref().map(|id| format!(" id=\"{}\"", id)).unwrap_or_default(),
                    if elem.class_list.is_empty() { 
                        String::new() 
                    } else { 
                        format!(" class=\"{}\"", elem.class_list.join(" "))
                    }
                ));

                if !elem.data_attributes.is_empty() {
                    report.push_str("    Data attributes: ");
                    for (key, val) in &elem.data_attributes {
                        report.push_str(&format!("{}=\"{}\" ", key, val));
                    }
                    report.push_str("\n");
                }

                if let Some(parent) = &elem.parent_info {
                    report.push_str(&format!("    Parent: {}\n", parent));
                }
            }
        }

        if let Some(shadow_roots) = results.dom_scan.get("shadow_roots") {
            report.push_str("\nElements with Shadow DOM:\n");
            for elem in shadow_roots {
                report.push_str(&format!("  - <{}{}>\n",
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
        options.child_list(true)
            .subtree(true)
            .attributes(true);

        let document = window().unwrap().document().unwrap();
        observer.observe(&document.body().unwrap(), &options)?;

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
