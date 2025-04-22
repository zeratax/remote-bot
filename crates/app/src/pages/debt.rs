use codee::string::JsonSerdeCodec;
use leptos::{
    IntoView, component, ev, html,
    logging::log,
    prelude::{
        ClassAttribute, Effect, ElementChild, Get, GetUntracked, GlobalAttributes, IntoAny,
        NodeRef, NodeRefAttribute, OnAttribute, PropAttribute, Set, event_target_value, signal,
    },
    view,
};
use leptos_use::storage::{UseStorageOptions, use_local_storage_with_options};

#[cfg(feature = "hydrate")]
use {
    gloo_timers::callback::Timeout,
    js_sys,
    leptos::prelude::Update,
    wasm_bindgen::{JsCast, closure::Closure},
    web_sys,
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "hydrate")]
const ANIMATION_DURATION: u32 = 500;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct GameState {
    debt: f64,
    goal: f64,
    minimum: u32,
    interest_rate: f64,
    penalty_rate: f64,
    last_update: u64,
    counter: u32,
    win_enabled: bool,
    update_interval: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
enum GameMessage {
    StateUpdate(GameState),
    CounterIncrement,
    GameReset(f64),
}

#[cfg(feature = "hydrate")]
fn current_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0).floor() as u64
}

#[cfg(feature = "hydrate")]
fn show_notification(title: &str, body: &str) {
    log!("Attempting to show notification");
    if let Some(window) = web_sys::window() {
        if let Ok(notification) =
            js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("Notification"))
        {
            if notification.is_undefined() {
                log!("Notifications not supported or not available.");
                return;
            }
            let permission = js_sys::Reflect::get(
                &notification,
                &wasm_bindgen::JsValue::from_str("permission"),
            )
            .unwrap_or(wasm_bindgen::JsValue::from_str("denied"));
            if permission.as_string().unwrap_or_default() != "granted" {
                log!("Notification permission not granted.");
                return;
            }
            let options = js_sys::Object::new();
            let _ = js_sys::Reflect::set(
                &options,
                &wasm_bindgen::JsValue::from_str("body"),
                &wasm_bindgen::JsValue::from_str(body),
            );
            let _ = js_sys::Reflect::set(
                &options,
                &wasm_bindgen::JsValue::from_str("icon"),
                &wasm_bindgen::JsValue::from_str("/favicon.ico"),
            );
            if let Some(constructor) = notification.dyn_ref::<js_sys::Function>() {
                let js_func = js_sys::Function::new_with_args(
                    "title,options",
                    "return new this(title, options)",
                );
                let _ = js_func
                    .call2(
                        &constructor,
                        &wasm_bindgen::JsValue::from_str(title),
                        &options,
                    )
                    .expect("Failed to create Notification");
            } else {
                log!("Notification constructor not a function.");
            }
        } else {
            log!("'Notification' not found on window object.");
        }
    } else {
        log!("Window object not available.");
    }
}

#[cfg(feature = "hydrate")]
fn get_share_url(goal: f64, update_interval_minutes: u32) -> String {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        if let Ok(href) = location.href() {
            let base_url = href.split('?').next().unwrap_or(&href);
            return format!(
                "{}?goal={}&interval={}",
                base_url, goal, update_interval_minutes
            );
        }
    }
    format!("?goal={}&interval={}", goal, update_interval_minutes)
}

#[cfg(feature = "hydrate")]
fn parse_url_params() -> Option<(f64, u32)> {
    if let Some(window) = web_sys::window() {
        let location = window.location();
        if let Ok(search) = location.search() {
            if !search.is_empty() {
                let params = search.trim_start_matches('?');
                let mut goal = None;
                let mut interval = None;

                for param in params.split('&') {
                    let parts: Vec<&str> = param.split('=').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "goal" => {
                                if let Ok(value) = parts[1].parse::<f64>() {
                                    goal = Some(value);
                                }
                            }
                            "interval" => {
                                if let Ok(value) = parts[1].parse::<u32>() {
                                    interval = Some(value);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if let (Some(g), Some(i)) = (goal, interval) {
                    return Some((g, i));
                }
            }
        }
    }
    None
}

#[cfg(feature = "hydrate")]
fn copy_to_clipboard(text: &str) -> bool {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        if let Ok(clipboard) =
            js_sys::Reflect::get(&navigator, &wasm_bindgen::JsValue::from_str("clipboard"))
        {
            if !clipboard.is_undefined() {
                if let Ok(write_text) =
                    js_sys::Reflect::get(&clipboard, &wasm_bindgen::JsValue::from_str("writeText"))
                {
                    if let Some(write_fn) = write_text.dyn_ref::<js_sys::Function>() {
                        let _ = write_fn.call1(&clipboard, &wasm_bindgen::JsValue::from_str(text));
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(feature = "hydrate")]
mod game_options {
    pub struct GameConstants {
        pub minimum_options: &'static [u32],
        pub interest_options: &'static [f64],
        pub penalty_options: &'static [f64],
    }

    pub static MINIMUM_OPTIONS: [u32; 5] = [10, 20, 30, 100, 200];
    pub static INTEREST_OPTIONS: [f64; 5] = [0.02, 0.05, 0.08, 0.10, 0.20];
    pub static PENALTY_OPTIONS: [f64; 6] = [0.05, 0.08, 0.12, 0.20, 0.30, 0.50];

    pub static GAME_CONSTANTS: GameConstants = GameConstants {
        minimum_options: &MINIMUM_OPTIONS,
        interest_options: &INTEREST_OPTIONS,
        penalty_options: &PENALTY_OPTIONS,
    };
}

#[component]
pub fn DebtGame() -> impl IntoView {
    let initial_state = GameState {
        debt: 0.0,
        goal: 0.0,
        minimum: 1,
        interest_rate: 0.02,
        penalty_rate: 0.05,
        last_update: 0,
        counter: 0,
        win_enabled: false,
        update_interval: 600000,
    };

    #[allow(unused_variables)]
    let (stored_state, set_stored_state, remove_stored_state) =
        use_local_storage_with_options::<Option<GameState>, JsonSerdeCodec>(
            "debt_game_state",
            UseStorageOptions::default()
                .delay_during_hydration(true)
                .initial_value(None),
        );

    let initial_derived_state = stored_state
        .get_untracked()
        .unwrap_or(initial_state.clone());

    let (debt, set_debt) = signal(initial_derived_state.debt);
    let (goal, set_goal) = signal(initial_derived_state.goal);
    let (minimum, set_minimum) = signal(initial_derived_state.minimum);
    let (interest_rate, set_interest_rate) = signal(initial_derived_state.interest_rate);
    let (penalty_rate, set_penalty_rate) = signal(initial_derived_state.penalty_rate);
    let (counter, set_counter) = signal(initial_derived_state.counter);
    let (win_enabled, set_win_enabled) = signal(initial_derived_state.win_enabled);
    let (update_interval, set_update_interval) = signal(initial_derived_state.update_interval);

    let (is_setup, set_is_setup) = signal(initial_derived_state.goal <= 0.0);
    let (new_goal, set_new_goal) = signal(1000.0);
    let (new_update_interval_minutes, set_new_update_interval_minutes) = signal(10);
    #[allow(unused_variables)]
    let (animation_active, set_animation_active) = signal(false);
    #[allow(unused_variables)]
    let (time_until_update, set_time_until_update) = signal(0);
    #[allow(unused_variables)]
    let (share_url_copied, set_share_url_copied) = signal(false);

    let counter_btn_ref = NodeRef::<html::Div>::new();
    let debt_display_ref = NodeRef::<html::Div>::new();

    #[cfg(feature = "hydrate")]
    let send_state_update = move || {
        let current_state = GameState {
            debt: debt.get_untracked(),
            goal: goal.get_untracked(),
            minimum: minimum.get_untracked(),
            interest_rate: interest_rate.get_untracked(),
            penalty_rate: penalty_rate.get_untracked(),
            last_update: current_timestamp(),
            counter: counter.get_untracked(),
            win_enabled: win_enabled.get_untracked(),
            update_interval: update_interval.get_untracked(),
        };
        log!("Saving state: {:?}", current_state);
        set_stored_state.set(Some(current_state));
    };

    #[cfg(feature = "hydrate")]
    let animate_counter_click = move || {
        log!("Animating counter click");
        if let Some(btn) = counter_btn_ref.get() {
            let _ = btn.class_list().add_1("scale-95");
            let handle = Timeout::new(100, move || {
                if let Some(b) = counter_btn_ref.get() {
                    let _ = b.class_list().remove_1("scale-95");
                }
            });
            handle.forget();
        }
    };

    #[cfg(feature = "hydrate")]
    let remove_stored_state_clone_for_roll = remove_stored_state.clone();
    #[cfg(feature = "hydrate")]
    let handle_roll_logic = move || {
        log!("Handle roll logic called");
        if js_sys::Math::random() < 0.5 {
            log!("Roll: Win!");
            set_goal.set(0.0);
            set_win_enabled.set(false);
            show_notification(
                "You won the debt game!",
                "Congratulations! You're debt free!",
            );
            remove_stored_state_clone_for_roll();
            log!("State removed from storage on win.");
        } else {
            log!("Roll: Continue!");
            let increased_debt = debt.get_untracked() + 200.0;
            set_debt.set(increased_debt);
            set_win_enabled.set(false);
            show_notification(
                "The debt increases...",
                &format!(
                    "Bad luck! Your debt increased by 200. Current debt: {:.2}",
                    increased_debt
                ),
            );
            send_state_update();
        }
    };

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            let current_debt = debt.get();
            if current_debt <= 0.0 {
                log!("Debt signal reached zero, triggering win animation.");
                if let Some(display) = debt_display_ref.get() {
                    let _ = display.class_list().add_1("animate-bounce");
                    let handle = Timeout::new(2000, move || {
                        if let Some(d) = debt_display_ref.get() {
                            let _ = d.class_list().remove_1("animate-bounce");
                        }
                    });
                    handle.forget();
                }
            }
        }
    });

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            let btn_el = counter_btn_ref.get();
            let display_el = debt_display_ref.get();

            if btn_el.is_some() && display_el.is_some() {
                log!(
                    "Element refs are available, setting up update interval Effect body (runs client-side once)"
                );
                if let Some(win) = web_sys::window() {
                    let stored_state_clone = stored_state.clone();
                    let set_time_until_update_clone = set_time_until_update.clone();
                    let update_interval_clone = update_interval.clone();
                    let update_callback = Closure::wrap(Box::new(move || {
                        let last_update_from_storage = stored_state_clone
                            .get_untracked()
                            .map_or(0, |state| state.last_update);
                        let now = current_timestamp();
                        let elapsed_secs = now.saturating_sub(last_update_from_storage);
                        let current_update_interval = update_interval_clone.get_untracked();
                        let total_update_interval_secs = current_update_interval as u64 / 1000;

                        if elapsed_secs >= total_update_interval_secs {
                            log!(
                                "Update interval reached (elapsed {}s), applying interest.",
                                elapsed_secs
                            );
                            log!("Apply interest logic called");
                            if is_setup.get_untracked() || debt.get_untracked() <= 0.0 {
                                log!("Apply interest skipped (setup or debt paid)");
                                return;
                            }

                            let current_minimum = minimum.get_untracked();
                            let current_interest = interest_rate.get_untracked();
                            let current_penalty = penalty_rate.get_untracked();
                            let current_counter = counter.get_untracked();
                            let current_debt = debt.get_untracked();

                            let interest_amount = current_debt * current_interest;
                            let debt_after_interest = current_debt + interest_amount;
                            set_debt.set(debt_after_interest);
                            log!("Applied interest: {:.2}", interest_amount);

                            let mut final_debt = debt_after_interest;
                            let mut notification_body = format!(
                                "Regular interest: {}%. Current debt: {:.2}",
                                (current_interest * 100.0).round() as u32,
                                debt_after_interest
                            );
                            let mut notification_title = "Interest Applied";

                            if current_counter < current_minimum {
                                let penalty_amount = debt_after_interest * current_penalty;
                                final_debt += penalty_amount;
                                set_debt.set(final_debt);
                                log!("Applied penalty: {:.2}", penalty_amount);
                                notification_title = "Penalty Applied!";
                                notification_body = format!(
                                    "You didn't reach the minimum of {}. Penalty: {}%. Current debt: {:.2}",
                                    current_minimum,
                                    (current_penalty * 100.0).round() as u32,
                                    final_debt
                                );
                            } else {
                                log!("Minimum met, no penalty.");
                            }

                            show_notification(notification_title, &notification_body);

                            set_counter.set(0);

                            // TODO: combine penalty login in one function
                            let game_constants = &game_options::GAME_CONSTANTS;
                            let min_idx = (js_sys::Math::random()
                                * game_constants.minimum_options.len() as f64)
                                .floor() as usize;
                            let int_idx = (js_sys::Math::random()
                                * game_constants.interest_options.len() as f64)
                                .floor() as usize;
                            let pen_idx = (js_sys::Math::random()
                                * game_constants.penalty_options.len() as f64)
                                .floor() as usize;

                            set_minimum.set(game_constants.minimum_options[min_idx]);
                            set_interest_rate.set(game_constants.interest_options[int_idx]);
                            set_penalty_rate.set(game_constants.penalty_options[pen_idx]);

                            set_animation_active.set(true);
                            let handle = Timeout::new(ANIMATION_DURATION, move || {
                                set_animation_active.set(false);
                            });
                            handle.forget();
                            log!("Parameters randomized.");
                            send_state_update();
                        } else {
                            let remaining = total_update_interval_secs - elapsed_secs;
                            set_time_until_update_clone.set(remaining as i32);
                            log!("Time until next update: {}s", remaining);
                        }
                    }) as Box<dyn Fn()>);

                    let _ = win
                        .set_interval_with_callback_and_timeout_and_arguments_0(
                            update_callback.as_ref().unchecked_ref(),
                            1000,
                        )
                        .expect("Failed to set interval");

                    update_callback.forget();
                } else {
                    log!("Window not available for setting interval");
                }
            }
        }
    });

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Requesting notification permission Effect body (runs client-side once)");
            if let Some(window) = web_sys::window() {
                if let Ok(notification) =
                    js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("Notification"))
                {
                    if notification.is_undefined() {
                        log!("Notification API not available.");
                        return;
                    }
                    let permission = js_sys::Reflect::get(
                        &notification,
                        &wasm_bindgen::JsValue::from_str("permission"),
                    )
                    .unwrap_or(wasm_bindgen::JsValue::from_str("denied"));
                    if permission.as_string().unwrap_or_default() == "default" {
                        log!("Permission is default, requesting.");
                        if let Ok(request_fn) = js_sys::Reflect::get(
                            &notification,
                            &wasm_bindgen::JsValue::from_str("requestPermission"),
                        ) {
                            if let Some(request_fn) = request_fn.dyn_ref::<js_sys::Function>() {
                                let _ = request_fn
                                    .call0(&notification)
                                    .expect("Failed to request notification permission");
                            } else {
                                log!("requestPermission is not a function.");
                            }
                        } else {
                            log!("requestPermission not found on Notification.");
                        }
                    } else {
                        log!(
                            "Permission is already {}",
                            permission.as_string().unwrap_or_default()
                        );
                    }
                } else {
                    log!("Notification not found on window.");
                }
            } else {
                log!("Window not available for notification permission.");
            }
        }
    });

    Effect::new(move |_| {
        if let Some(state) = stored_state.get() {
            log!("Stored state signal updated, updating game state signals.");
            set_debt.set(state.debt);
            set_goal.set(state.goal);
            set_minimum.set(state.minimum);
            set_interest_rate.set(state.interest_rate);
            set_penalty_rate.set(state.penalty_rate);
            set_counter.set(state.counter);
            set_win_enabled.set(state.win_enabled);
            set_update_interval.set(state.update_interval);
        } else {
            log!(
                "Stored state signal is None (e.g., on reset), resetting game state signals to initial default."
            );
            set_debt.set(initial_state.debt);
            set_goal.set(initial_state.goal);
            set_minimum.set(initial_state.minimum);
            set_interest_rate.set(initial_state.interest_rate);
            set_penalty_rate.set(initial_state.penalty_rate);
            set_counter.set(initial_state.counter);
            set_win_enabled.set(initial_state.win_enabled);
            set_update_interval.set(initial_state.update_interval);
        }
        set_is_setup.set(goal.get_untracked() <= 0.0);
        log!("is_setup updated to: {}", is_setup.get_untracked());
    });

    let on_share_setup = move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Share setup button clicked");
            let url = get_share_url(
                new_goal.get_untracked(),
                new_update_interval_minutes.get_untracked(),
            );

            if copy_to_clipboard(&url) {
                log!("URL copied to clipboard: {}", url);
                set_share_url_copied.set(true);

                let handle = Timeout::new(3000, move || {
                    set_share_url_copied.set(false);
                });
                handle.forget();
            } else {
                log!("Failed to copy URL to clipboard");
            }
        }
    };

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Checking URL parameters for shared game setup");
            if let Some((url_goal, url_interval_minutes)) = parse_url_params() {
                log!(
                    "Found URL parameters: goal={}, interval={}",
                    url_goal,
                    url_interval_minutes
                );
                set_new_goal.set(url_goal);
                set_new_update_interval_minutes.set(url_interval_minutes);
            }
        }
    });

    let format_time_remaining = move || {
        let remaining = time_until_update.get();
        let minutes = remaining / 60;
        let seconds = remaining % 60;
        format!("{:02}:{:02}", minutes, seconds)
    };

    let format_currency = |amount: f64| format!("${:.2}", amount);
    let format_percentage = |rate: f64| format!("{}%", (rate * 100.0).round() as u32);

    let format_interval = |interval_ms: u32| format!("{} min", interval_ms / 60000);

    #[allow(unused_variables)]
    let on_setup_submit = move |ev: ev::SubmitEvent| {
        #[cfg(feature = "hydrate")]
        {
            ev.prevent_default();
            let new_goal_value = new_goal.get_untracked();
            let new_interval_minutes = new_update_interval_minutes.get_untracked();

            let new_interval_ms = new_interval_minutes as u32 * 60000;

            log!(
                "Setup submit: New goal = {}, interval = {} minutes",
                new_goal_value,
                new_interval_minutes
            );

            set_goal.set(new_goal_value);
            set_debt.set(new_goal_value);
            set_counter.set(0);
            set_win_enabled.set(false);
            set_update_interval.set(new_interval_ms);

            let game_constants = &game_options::GAME_CONSTANTS;
            let min_idx = (js_sys::Math::random() * game_constants.minimum_options.len() as f64)
                .floor() as usize;
            let int_idx = (js_sys::Math::random() * game_constants.interest_options.len() as f64)
                .floor() as usize;
            let pen_idx = (js_sys::Math::random() * game_constants.penalty_options.len() as f64)
                .floor() as usize;

            set_minimum.set(game_constants.minimum_options[min_idx]);
            set_interest_rate.set(game_constants.interest_options[int_idx]);
            set_penalty_rate.set(game_constants.penalty_options[pen_idx]);

            set_animation_active.set(true);
            let handle = Timeout::new(ANIMATION_DURATION, move || {
                set_animation_active.set(false);
            });
            handle.forget();
            log!("Parameters randomized.");
            send_state_update();
        }
    };

    let on_increment = move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Increment click handler called");
            if !is_setup.get_untracked() && debt.get_untracked() > 0.0 {
                set_counter.update(|c| *c += 1);
                set_debt.update(|d| *d -= 1.0);

                if debt.get_untracked() <= 0.0 {
                    log!("Debt reached zero, enabling win.");
                    set_win_enabled.set(true);
                }

                animate_counter_click();
                send_state_update();
            } else {
                log!(
                    "Increment click ignored: is_setup={}, debt={}",
                    is_setup.get_untracked(),
                    debt.get_untracked()
                );
            }
        }
    };

    let on_roll = move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Roll click handler called");
            if win_enabled.get_untracked() {
                handle_roll_logic();
            } else {
                log!(
                    "Roll click ignored: win_enabled={}",
                    win_enabled.get_untracked()
                );
            }
        }
    };

    let remove_stored_state_clone_for_reset = remove_stored_state.clone();
    let on_reset_game = move |_| {
        log!("Reset game handler called");
        set_debt.set(initial_state.debt);
        set_goal.set(initial_state.goal);
        set_minimum.set(initial_state.minimum);
        set_interest_rate.set(initial_state.interest_rate);
        set_penalty_rate.set(initial_state.penalty_rate);
        set_counter.set(initial_state.counter);
        set_win_enabled.set(initial_state.win_enabled);
        set_update_interval.set(initial_state.update_interval);

        log!("Resetting state in storage");
        remove_stored_state_clone_for_reset();
    };

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl p-6 max-w-2xl mx-auto transition-all">
            {move || if is_setup.get() {
                view! {
                    <div class="flex flex-col items-center space-y-6 py-4">
                        <h2 class="text-3xl font-bold text-center mb-4">"Debt Challenge Game"</h2>
                        <p class="text-gray-600 dark:text-gray-300 text-center max-w-md">
                            "Set your debt goal and try to reduce it to zero before interests and penalties overwhelm you!"
                        </p>

                        <form class="w-full max-w-sm" on:submit=on_setup_submit.clone()>
                            <div class="mb-6">
                                <label for="goal-input" class="block text-sm font-medium mb-2">
                                    "Set your initial debt:"
                                </label>
                                <div class="flex items-center">
                                    <span class="text-gray-500 dark:text-gray-400 mr-2 text-xl">"$"</span>
                                    <input
                                        id="goal-input"
                                        type="number"
                                        min="100"
                                        max="10000"
                                        step="100"
                                        prop:value=new_goal.get()
                                        on:input=move |ev| {
                                            let value = event_target_value(&ev).parse::<f64>().unwrap_or(1000.0);
                                            set_new_goal.set(value.max(0.0));
                                        }
                                        class="flex-1 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md py-3 px-4 text-xl text-right transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    />
                                </div>
                            </div>

                            <div class="mb-6">
                                <label for="interval-input" class="block text-sm font-medium mb-2">
                                    "Set interest cycle duration:"
                                </label>
                                <div class="flex items-center">
                                    <select
                                        id="interval-input"
                                        prop:value=new_update_interval_minutes.get()
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev).parse::<u32>().unwrap_or(10);
                                            set_new_update_interval_minutes.set(value);
                                        }
                                        class="flex-1 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md py-3 px-4 text-xl transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    >
                                        <option value="1">"1 minute"</option>
                                        <option value="3">"3 minutes"</option>
                                        <option value="5">"5 minutes"</option>
                                        <option value="10" selected>"10 minutes"</option>
                                        <option value="30">"30 minutes"</option>
                                        <option value="60">"1 hour"</option>
                                        <option value="1440">"1 day"</option>
                                    </select>
                                </div>
                                <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                    "Shorter intervals make the game more challenging"
                                </p>
                            </div>

                            <button
                                type="submit"
                                class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-4 rounded-md shadow-md transform hover:scale-105 transition-all mb-4"
                            >
                                "Start Game"
                            </button>

                            <div class="flex flex-col items-center">
                                <button
                                    type="button"
                                    class="inline-flex items-center px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-700 dark:text-gray-300 rounded-md text-sm font-medium transition-colors"
                                    on:click=on_share_setup
                                >
                                    <span class="mr-2">"🔗"</span>
                                    "Share this setup"
                                </button>

                                <div
                                    class={move || {
                                        let base = "text-xs text-green-600 dark:text-green-400 mt-2 transition-opacity";
                                        if share_url_copied.get() { format!("{} {}", base, "opacity-100") } else { format!("{} {}", base, "opacity-0") }
                                    }}
                                >
                                    "✓ Share link copied to clipboard!"
                                </div>
                            </div>
                        </form>

                        <div class="mt-6 text-sm text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 p-4 rounded-md">
                            <h3 class="font-bold mb-2">"How to play:"</h3>
                            <ul class="list-disc pl-5 space-y-1">
                                <li>"Reduce your debt to zero by clicking the counter button"</li>
                                <li>"Every cycle, interest is applied to your remaining debt"</li>
                                <li>"You must increase your counter by at least the minimum value before each interest cycle"</li>
                                <li>"Failure to reach the minimum results in a penalty interest rate"</li>
                                <li>"When you reach zero, you can roll for a chance to win or continue"</li>
                            </ul>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-2xl font-bold">"Debt Challenge"</h2>
                            <div class="bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 px-3 py-1 rounded-full text-sm font-medium">
                                "Next update: " {format_time_remaining}
                            </div>
                        </div>

                        <div
                            node_ref=debt_display_ref
                            class="bg-gray-100 dark:bg-gray-700 rounded-lg p-8 mb-6 text-center transition-all"
                            class:animate-pulse=animation_active
                        >
                            <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">"Remaining Debt:"</div>
                            <div class="text-4xl font-bold mb-2 transition-all">{move || format_currency(debt.get())}</div>
                            <div class="text-xs text-gray-500 dark:text-gray-400">
                                "Original goal: " {move || format_currency(goal.get())}
                            </div>
                            <div class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                "Interest cycle: " {move || format_interval(update_interval.get())}
                            </div>
                        </div>

                        <div class="grid grid-cols-3 gap-4 mb-6">
                            <div class={move || {
                                let base = "p-4 rounded-lg text-center transition-all";
                                if animation_active.get() { format!("{} {}", base, "bg-yellow-100 dark:bg-yellow-900") } else { format!("{} {}", base, "bg-gray-100 dark:bg-gray-700") }
                            }}>
                                <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">"Minimum"</div>
                                <div class="text-xl font-bold">{move || minimum.get().to_string()}</div>
                            </div>
                            <div class={move || {
                                let base = "p-4 rounded-lg text-center transition-all";
                                if animation_active.get() { format!("{} {}", base, "bg-green-100 dark:bg-green-900") } else { format!("{} {}", base, "bg-gray-100 dark:bg-gray-700") }
                            }}>
                                <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">"Interest"</div>
                                <div class="text-xl font-bold">{move || format_percentage(interest_rate.get())}</div>
                            </div>
                            <div class={move || {
                                let base = "p-4 rounded-lg text-center transition-all";
                                if animation_active.get() { format!("{} {}", base, "bg-red-100 dark:bg-red-900") } else { format!("{} {}", base, "bg-gray-100 dark:bg-gray-700") }
                            }}>
                                <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">"Penalty"</div>
                                <div class="text-xl font-bold">{move || format_percentage(penalty_rate.get())}</div>
                            </div>
                        </div>

                        <div class="flex items-center justify-between bg-gray-100 dark:bg-gray-700 rounded-lg p-4 mb-6">
                            <div>
                                <div class="text-sm text-gray-500 dark:text-gray-400">"Current Counter:"</div>
                                <div class="text-2xl font-bold">{move || counter.get().to_string()}
                                    <span class="text-sm text-gray-500 dark:text-gray-400 ml-1">
                                        "/ " {move || minimum.get().to_string()} " min"
                                    </span>
                                </div>
                            </div>
                            <div
                                node_ref=counter_btn_ref
                                class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-lg shadow-md transform hover:scale-105 active:scale-95 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                                class:opacity-50=move || debt.get() <= 0.0
                                on:click=on_increment
                            >
                                "Click to Reduce"
                            </div>
                        </div>

                        <div
                            class={move || {
                                let base = "flex justify-center transition-all";
                                if win_enabled.get() { format!("{} {}", base, "opacity-100 transform translate-y-0") } else { format!("{} {}", base, "opacity-0 transform -translate-y-4 pointer-events-none") }
                            }}
                        >
                            <button
                                class="mt-4 bg-yellow-500 hover:bg-yellow-600 text-white font-bold py-3 px-6 rounded-lg shadow-lg transform hover:scale-105 transition-all animate-pulse"
                                on:click=on_roll.clone()
                            >
                                "Roll For Your Fate!"
                            </button>
                        </div>

                        <div class="mt-6 text-center">
                            <button
                                class="text-sm text-gray-500 dark:text-gray-400 hover:text-blue-600 dark:hover:text-blue-400"
                                on:click=on_reset_game.clone()
                            >
                                "Reset Game"
                            </button>
                        </div>
                    </div>
                }.into_any()
            }
        }
        </div>
    }
}
