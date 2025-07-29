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
