use codee::string::JsonSerdeCodec;
use leptos::{
    IntoView, component, html,
    logging::log,
    prelude::{
        ClassAttribute, Effect, ElementChild, Get, GetUntracked, GlobalAttributes, IntoAny, Memo,
        NodeRef, NodeRefAttribute, OnAttribute, PropAttribute, Set, event_target_value, signal,
    },
    view,
};
use leptos_router::hooks::use_query_map;
use leptos_use::storage::{UseStorageOptions, use_local_storage_with_options};
use serde::{Deserialize, Serialize};

use crate::components::random_background_image::RandomBackgroundImage;

#[cfg(feature = "hydrate")]
use leptos::prelude::{Update, on_cleanup};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
struct GameState {
    debt: f64,
    goal: f64,
    initial_goal: f64,
    minimum: u32,
    interest_rate: f64,
    penalty_rate: f64,
    #[allow(unused_variables)] // Used in timer effect
    last_update: u64,
    counter: u32,
    total_paid: f64,
    win_enabled: bool,
    update_interval: u32, // in milliseconds
    is_won: bool,
}

const DEFAULT_STATE: GameState = GameState {
    debt: 0.0,
    goal: 0.0,
    initial_goal: 100.0,
    minimum: 0,
    interest_rate: 0.02,
    penalty_rate: 0.02,
    last_update: 0,
    counter: 0,
    total_paid: 0.0,
    win_enabled: false,
    update_interval: 600_000,
    is_won: false,
};

#[component]
pub fn DebtGame() -> impl IntoView {
    let (stored, set_stored, _remove_stored) =
        use_local_storage_with_options::<Option<GameState>, JsonSerdeCodec>(
            "debt_game_state",
            UseStorageOptions::default()
                .delay_during_hydration(true)
                .initial_value(None),
        );

    let query = use_query_map();
    let url_goal = query
        .get_untracked()
        .get("goal")
        .and_then(|s| s.parse::<f64>().ok());
    let url_interval_minutes = query
        .get_untracked()
        .get("interval")
        .and_then(|s| s.parse::<u32>().ok());

    let stored_state = stored.get_untracked();

    let init_state = match stored_state {
        Some(s) if s.goal > 0.0 => s,
        _ if url_goal.is_some() && url_interval_minutes.is_some() => {
            let g = url_goal.unwrap();
            let interval_ms = url_interval_minutes.unwrap() * 60_000;
            GameState {
                initial_goal: g,
                goal: g,
                debt: g,
                update_interval: interval_ms,
                ..DEFAULT_STATE
            }
        }
        _ => DEFAULT_STATE,
    };
    log!("Initial game state: {:?}", init_state);

    let (debt, set_debt) = signal(init_state.debt);
    let (goal, set_goal) = signal(init_state.goal);
    let (initial_goal, set_initial_goal) = signal(init_state.initial_goal);
    let (minimum, set_minimum) = signal(init_state.minimum);
    let (interest_rate, set_interest_rate) = signal(init_state.interest_rate);
    let (penalty_rate, set_penalty_rate) = signal(init_state.penalty_rate);
    let (counter, set_counter) = signal(init_state.counter);
    let (total_paid, set_total_paid) = signal(init_state.total_paid);
    let (win_enabled, set_win_enabled) = signal(init_state.win_enabled);
    let (update_interval, set_update_interval) = signal(init_state.update_interval);
    #[allow(unused_variables)] // Used in timer effect
    let (last_update, set_last_update) = signal(init_state.last_update);
    #[allow(unused_variables)] // Used in share handler
    let (share_url_copied, set_share_url_copied) = signal(false);
    #[allow(unused_variables)] // Used in timer display
    let (time_until_update, set_time_until_update) = signal(0_i32);
    #[allow(unused_variables)] // Used for parameter change animation class
    let (animation_active, set_animation_active) = signal(false);
    #[allow(unused_variables)] // Used for win animation class
    let (win_animation_active, set_win_animation_active) = signal(false);
    let (is_won, set_is_won) = signal(init_state.is_won);

    let is_setup = Memo::new(move |_| goal.get() <= 0.0);

    let formatted_cycle = Memo::new(move |_| {
        let secs = time_until_update.get();
        let m = secs / 60;
        let s = secs % 60;
        format!("{:02}:{:02}", m, s)
    });

    let debt_text_color_class = Memo::new(move |_| {
        if debt.get() > 0.0 && counter.get() < minimum.get() {
            "text-red-600 dark:text-red-400"
        } else {
            "text-gray-900 dark:text-gray-100"
        }
    });

    let debt_display_ref = NodeRef::<html::Div>::new();
    let counter_btn_ref = NodeRef::<html::Button>::new();

    #[allow(unused_variables)] // Used in apply_cycle and on_click
    let persist = move |ts: u64| {
        let s = GameState {
            debt: debt.get_untracked(),
            goal: goal.get_untracked(),
            initial_goal: initial_goal.get_untracked(),
            minimum: minimum.get_untracked(),
            interest_rate: interest_rate.get_untracked(),
            penalty_rate: penalty_rate.get_untracked(),
            last_update: ts,
            counter: counter.get_untracked(),
            total_paid: total_paid.get_untracked(),
            win_enabled: win_enabled.get_untracked(),
            update_interval: update_interval.get_untracked(),
            is_won: is_won.get_untracked(),
        };
        set_stored.set(Some(s));
        log!("Game state persisted: {:?}", s.clone());
    };

    #[allow(unused_variables)] // Used in apply_cycle and on_setup
    let randomize_params = move || {
        #[cfg(feature = "hydrate")]
        {
            let opts = &crate::components::game_options::GAME_CONSTANTS;
            let min_idx =
                (js_sys::Math::random() * opts.minimum_options.len() as f64).floor() as usize;
            let int_idx =
                (js_sys::Math::random() * opts.interest_options.len() as f64).floor() as usize;
            let pen_idx =
                (js_sys::Math::random() * opts.penalty_options.len() as f64).floor() as usize;

            set_minimum.set(opts.minimum_options[min_idx]);
            set_interest_rate.set(opts.interest_options[int_idx]);
            set_penalty_rate.set(opts.penalty_options[pen_idx]);

            set_animation_active.set(true);
            let handle = gloo_timers::callback::Timeout::new(500, move || {
                set_animation_active.set(false);
            });
            handle.forget();

            log!("Parameters randomized.");
        }
    };

    #[allow(unused_variables)] // Used in timer effect
    let apply_cycle = move || {
        #[cfg(feature = "hydrate")]
        {
            let now = js_sys::Date::now().floor() as u64;
            if !is_setup.get_untracked() && debt.get_untracked() > 0.0 && !is_won.get_untracked() {
                log!("Applying interest/penalty cycle.");
                let base_debt = debt.get_untracked();
                let interest = base_debt * interest_rate.get_untracked();
                let mut new_debt = base_debt + interest;
                let mut notification_body = format!(
                    "Interest {}%: +{:.2}",
                    (interest_rate.get_untracked() * 100.0).round(),
                    interest
                );
                let mut notification_title = "Interest Applied";

                if counter.get_untracked() < minimum.get_untracked() {
                    let penalty = new_debt * penalty_rate.get_untracked();
                    new_debt += penalty;
                    notification_title = "Penalty Applied!";
                    notification_body = format!(
                        "Minimum not met! Penalty {}%: +{:.2}. {}",
                        (penalty_rate.get_untracked() * 100.0).round(),
                        penalty,
                        notification_body
                    );
                } else {
                    log!("Minimum met, no penalty.");
                }

                set_debt.set(new_debt);
                set_counter.set(0);
                randomize_params();
                persist(now);
                set_last_update.set(now);
                crate::util::notifications::show_notification(
                    notification_title,
                    &notification_body,
                );
            } else {
                log!(
                    "Cycle skipped: is_setup={}, debt={}, is_won={}",
                    is_setup.get_untracked(),
                    debt.get_untracked(),
                    is_won.get_untracked()
                );
            }
        }
    };

    Effect::new(move |_| {
        if let Some(s) = stored.get() {
            log!("Loading state from storage effect triggered.");
            set_debt.set(s.debt);
            set_goal.set(s.goal);
            set_initial_goal.set(s.initial_goal);
            set_minimum.set(s.minimum);
            set_interest_rate.set(s.interest_rate);
            set_penalty_rate.set(s.penalty_rate);
            set_counter.set(s.counter);
            set_total_paid.set(s.total_paid);
            set_win_enabled.set(s.win_enabled);
            set_update_interval.set(s.update_interval);
            set_last_update.set(s.last_update);
            set_is_won.set(s.is_won);
        }
    });

    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            let apply_cycle = apply_cycle.clone();
            let last_update = last_update.clone();
            let update_interval = update_interval.clone();
            let set_time_until_update = set_time_until_update.clone();
            let is_setup = is_setup.clone();
            let is_won = is_won.clone();

            log!("Setting up 1-second interval timer.");
            let interval = send_wrapper::SendWrapper::new(gloo_timers::callback::Interval::new(
                1_000,
                move || {
                    if is_setup.get_untracked() || is_won.get_untracked() {
                        set_time_until_update((update_interval.get_untracked() / 1000) as i32);
                        return;
                    }

                    let now = js_sys::Date::now().floor() as u64;
                    let elapsed_ms = now.saturating_sub(last_update.get_untracked());
                    let interval_ms = update_interval.get_untracked() as u64;

                    if elapsed_ms < interval_ms {
                        let remaining_ms = interval_ms - elapsed_ms;
                        set_time_until_update((remaining_ms / 1000).max(0) as i32);
                    } else {
                        apply_cycle();
                        set_time_until_update((update_interval.get_untracked() / 1000) as i32);
                    }
                },
            ));
            on_cleanup(move || {
                log!("Cleaning up 1-second interval timer.");
                drop(interval)
            });
        }
    });

    // Effect to trigger win animation when debt reaches zero
    Effect::new(move |_| {
        #[cfg(feature = "hydrate")]
        {
            let current_debt = debt.get();
            if current_debt <= 0.0 && goal.get() > 0.0 && !is_won.get_untracked() {
                log!("Debt signal reached zero, triggering win state.");
                set_win_enabled.set(true);
                set_win_animation_active.set(true);
            } else {
                if debt.get() > 0.0 || goal.get() <= 0.0 || is_won.get_untracked() {
                    set_win_enabled.set(false);
                }
                if !is_won.get_untracked() {
                    set_win_animation_active.set(false);
                }
            }
        }
    });

    let format_pct = |r: f64| format!("{}%", (r * 100.0).round() as u32);

    #[allow(unused_variables)] // Used in setup form
    let on_setup = move |#[allow(unused_variables)] ev: leptos::ev::SubmitEvent| {
        #[cfg(feature = "hydrate")]
        {
            ev.prevent_default();
            let start = js_sys::Date::now().floor() as u64;
            let initial_goal_value = initial_goal.get_untracked();

            if initial_goal_value <= 0.0 {
                log!("Setup failed: Initial goal must be positive.");
                return;
            }

            set_goal.set(initial_goal_value);
            set_debt.set(initial_goal_value);
            set_total_paid.set(0.0);
            set_last_update.set(start);
            set_counter.set(0);
            set_is_won.set(false);
            randomize_params();
            persist(start);
            log!("Game setup complete.");
        }
    };

    #[allow(unused_variables)] // Used in share button
    let on_share = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if let Some(window) = web_sys::window() {
                let location = window.location();
                if let Ok(href) = location.href() {
                    let base = href.split('?').next().unwrap_or(&href);
                    let url = format!(
                        "{}?goal={}&interval={}",
                        base,
                        initial_goal.get_untracked(),
                        update_interval.get_untracked() / 60_000
                    );
                    if crate::util::clipboard::copy_to_clipboard(&url) {
                        set_share_url_copied.set(true);
                        let s = set_share_url_copied.clone();
                        gloo_timers::callback::Timeout::new(3000, move || s.set(false)).forget();
                        log!("Share URL copied: {}", url);
                    } else {
                        log!("Failed to copy share URL.");
                    }
                }
            }
        }
    };

    #[allow(unused_variables)] // Used in click button
    let on_click = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if !is_setup.get_untracked() && debt.get_untracked() > 0.0 && !is_won.get_untracked() {
                set_counter.update(|c| *c += 1);
                set_debt.update(|d| *d -= 1.0);
                set_total_paid.update(|t| *t += 1.0);

                if let Some(btn) = counter_btn_ref.get_untracked() {
                    let _ = btn.class_list().add_1("scale-95");
                    let handle = gloo_timers::callback::Timeout::new(100, move || {
                        if let Some(b) = counter_btn_ref.get_untracked() {
                            let _ = b.class_list().remove_1("scale-95");
                        }
                    });
                    handle.forget();
                }

                persist(last_update.get_untracked());
                log!(
                    "Click applied. Counter: {}, Debt: {:.2}",
                    counter.get_untracked(),
                    debt.get_untracked()
                );
            } else {
                log!(
                    "Click ignored: is_setup={}, debt={}, is_won={}",
                    is_setup.get_untracked(),
                    debt.get_untracked(),
                    is_won.get_untracked()
                );
            }
        }
    };

    let on_roll = move |_| {
        #[cfg(feature = "hydrate")]
        {
            if win_enabled.get_untracked() && !is_won.get_untracked() {
                log!("Roll button clicked. Performing roll logic.");
                let now = js_sys::Date::now().floor() as u64;
                if js_sys::Math::random() < 0.5 {
                    log!("Roll: Win!");
                    set_is_won.set(true);
                    set_win_enabled.set(false);
                    persist(now);
                    set_last_update.set(now);
                    crate::util::notifications::show_notification("You Won!", "You are debt free!");
                } else {
                    log!("Roll: Lose! Debt increases.");
                    let increased_amount = 200.0;
                    set_debt.update(|d| *d += increased_amount);
                    set_win_enabled.set(false);

                    persist(now);
                    set_last_update.set(now);

                    crate::util::notifications::show_notification(
                        "Bad Luck!",
                        &format!("Your debt increased by {}", increased_amount.round()),
                    );
                }
            } else {
                log!(
                    "Roll button ignored: win_enabled={}, is_won={}",
                    win_enabled.get_untracked(),
                    is_won.get_untracked()
                );
            }
        }
    };

    let on_reset = move |_| {
        #[cfg(feature = "hydrate")]
        {
            log!("Resetting game.");
            set_stored.set(None);

            // Reset signals to default state
            set_debt.set(DEFAULT_STATE.debt);
            set_goal.set(DEFAULT_STATE.goal);
            set_initial_goal.set(DEFAULT_STATE.initial_goal);
            set_minimum.set(DEFAULT_STATE.minimum);
            set_interest_rate.set(DEFAULT_STATE.interest_rate);
            set_penalty_rate.set(DEFAULT_STATE.penalty_rate);
            set_counter.set(DEFAULT_STATE.counter);
            set_total_paid.set(DEFAULT_STATE.total_paid);
            set_win_enabled.set(DEFAULT_STATE.win_enabled);
            set_update_interval.set(DEFAULT_STATE.update_interval);
            set_last_update.set(DEFAULT_STATE.last_update);
            set_is_won.set(DEFAULT_STATE.is_won);
            log!("Game state reset.");
        }
    };

    view! {
        <RandomBackgroundImage>
            <div class="min-h-screen w-full pt-16 px-8">
                <div class="bg-white/30 dark:bg-gray-800/30 backdrop-blur-sm rounded-lg shadow-xl p-6 max-w-2xl mx-auto w-full transition-all duration-500 ease-in-out">
                    {move || if is_setup.get() {
                        view! {
                            <div class="flex flex-col items-center space-y-6 py-4">
                                <h2 class="text-3xl font-bold text-center mb-4">"Edging Challenge Game"</h2>
                                <p class="text-gray-700 dark:text-gray-300 text-center max-w-md">
                                    "Set your debt goal and try to reduce it to zero before interests and penalties overwhelm you!"
                                </p>

                                <form class="w-full max-w-sm" on:submit=on_setup.clone()>
                                    <div class="mb-6">
                                        <label for="goal-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                            "Set your initial edging debt:"
                                        </label>
                                        <div class="flex items-center">
                                            <span class="text-gray-600 dark:text-gray-400 mr-2 text-xl">"Edges"</span>
                                            <input
                                                id="goal-input"
                                                type="number"
                                                min="10"
                                                max="10000"
                                                step="10"
                                                prop:value=initial_goal.get()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev).parse::<f64>().unwrap_or(DEFAULT_STATE.initial_goal);
                                                    set_initial_goal.set(value.max(10.0));
                                                }
                                                class="flex-1 bg-white/50 dark:bg-gray-700/50 border border-gray-300 dark:border-gray-600 rounded-md py-3 px-4 text-xl text-right transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500"
                                            />
                                        </div>
                                    </div>

                                    <div class="mb-6">
                                        <label for="interval-input" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                            "Set interest cycle duration:"
                                        </label>
                                        <div class="flex items-center">
                                            <select
                                                id="interval-input"
                                                prop:value=update_interval.get()
                                                on:change=move |ev| {
                                                     let value = event_target_value(&ev).parse::<u32>().unwrap_or(DEFAULT_STATE.update_interval);
                                                     set_update_interval.set(value);
                                                }
                                                class="flex-1 bg-white/50 dark:bg-gray-700/50 border border-gray-300 dark:border-gray-600 rounded-md py-3 px-4 text-xl transition-colors focus:outline-none focus:ring-2 focus::ring-blue-500"
                                            >
                                                <option value="60000"    selected={update_interval.get_untracked() == 60_000}    >"1 minute"</option>
                                                <option value="180000"   selected={update_interval.get_untracked() == 180_000}   >"3 minutes"</option>
                                                <option value="300000"   selected={update_interval.get_untracked() == 300_000}   >"5 minutes"</option>
                                                <option value="600000"   selected={update_interval.get_untracked() == 600000}    >"10 minutes"</option>
                                                <option value="1800000"  selected={update_interval.get_untracked() == 1_800_000} >"30 minutes"</option>
                                                <option value="3600000"  selected={update_interval.get_untracked() == 3_600_000} >"1 hour"</option>
                                                <option value="86400000" selected={update_interval.get_untracked() == 86_400_000}>"1 day"</option>
                                            </select>
                                        </div>
                                        <p class="text-xs text-gray-600 dark:text-gray-400 mt-1">
                                            "Shorter intervals make the game more challenging"
                                        </p>
                                    </div>

                                    <button
                                        type="submit"
                                        class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-4 rounded-md shadow-md transform hover:scale-105 transition-all duration-300 mb-4"
                                    >
                                        "Start Game"
                                    </button>

                                    <div class="flex flex-col items-center">
                                        <button
                                            type="button"
                                            class="inline-flex items-center px-4 py-2 bg-gray-200/50 hover:bg-gray-300/50 dark:bg-gray-700/50 dark:hover:bg-gray-600/50 text-gray-800 dark:text-gray-200 rounded-md text-sm font-medium transition-colors duration-200"
                                            on:click=on_share
                                        >
                                            <span class="mr-2">"🔗"</span>
                                            "Share this setup"
                                        </button>

                                        <div
                                            class={move || {
                                                let base = "text-xs text-green-600 dark:text-green-400 mt-2 transition-opacity duration-300";
                                                if share_url_copied.get() { format!("{} {}", base, "opacity-100") } else { format!("{} {}", base, "opacity-0") }
                                            }}
                                        >
                                            "✓ Share link copied to clipboard!"
                                        </div>
                                    </div>
                                </form>

                                <div class="mt-6 text-sm text-gray-700 dark:text-gray-300 bg-white/50 dark:bg-gray-700/50 p-4 rounded-md">
                                    <h3 class="font-bold mb-2">"How to play:"</h3>
                                    <ul class="list-disc pl-5 space-y-1">
                                        <li>"Reduce your debt to zero by clicking the edge button."</li>
                                        <li>"Every cycle, interest is applied to your remaining debt."</li>
                                        <li>"You must increase your counter by at least the minimum value before each interest cycle."</li>
                                        <li>"Failure to reach the minimum results in a penalty interest rate."</li>
                                        <li>"When you reach zero debt, you can roll for a chance to cum or continue."</li>
                                    </ul>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="flex flex-col space-y-4">
                                <div class="flex justify-between items-center mb-2">
                                    <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">"Edging Debt Challenge"</h2>
                                    <div class="bg-blue-500/70 dark:bg-blue-800/70 text-white px-3 py-1 rounded-full text-sm font-medium backdrop-blur-sm">
                                        "Next update: " {move || formatted_cycle.get()}
                                    </div>
                                </div>

                                <div
                                    node_ref=debt_display_ref
                                    class={move || {
                                        let base = "bg-white/50 dark:bg-gray-700/50 rounded-lg p-6 text-center transition-all duration-500 ease-in-out backdrop-blur-sm";
                                        let animation = if animation_active.get() { "animate-pulse" } else { "" };
                                        let win_anim = if win_animation_active.get() { "animate-bounce" } else { "" };
                                        format!("{} {} {}", base, animation, win_anim)
                                    }}
                                >
                                    <div class="text-sm text-gray-700 dark:text-gray-300 mb-1">"Remaining Debt:"</div>
                                    <div class={move || format!("text-4xl font-bold transition-colors duration-300 {}", debt_text_color_class.get())}>
                                        {move || format!("{}", debt.get().round())}
                                    </div>
                                    <div class="text-xs text-gray-600 dark:text-gray-400 mt-2">
                                        "Original Edging Goal: " {move || format!("{} Edges", goal.get().round())}
                                         " | Total Edges: " {move || format!("{}", total_paid.get().round())}
                                    </div>
                                </div>

                                <div class="grid grid-cols-3 gap-4">
                                    <div class={move || {
                                        let base = "p-4 rounded-lg text-center transition-all duration-300 backdrop-blur-sm";
                                        if animation_active.get() { format!("{} {}", base, "bg-yellow-500/50 dark:bg-yellow-800/50") } else { format!("{} {}", base, "bg-white/50 dark:bg-gray-700/50") }
                                    }}>
                                        <div class="text-sm text-gray-700 dark:text-gray-300 mb-1">"Minimum"</div>
                                        <div class="text-xl font-bold text-gray-900 dark:text-gray-100">{move || minimum.get().to_string()}</div>
                                    </div>
                                    <div class={move || {
                                        let base = "p-4 rounded-lg text-center transition-all duration-300 backdrop-blur-sm";
                                        if animation_active.get() { format!("{} {}", base, "bg-green-500/50 dark:bg-green-800/50") } else { format!("{} {}", base, "bg-white/50 dark:bg-gray-700/50") }
                                    }}>
                                        <div class="text-sm text-gray-700 dark:text-gray-300 mb-1">"Interest"</div>
                                        <div class="text-xl font-bold text-gray-900 dark:text-gray-100">{move || format_pct(interest_rate.get())}</div>
                                    </div>
                                    <div class={move || {
                                        let base = "p-4 rounded-lg text-center transition-all duration-300 backdrop-blur-sm";
                                        if animation_active.get() { format!("{} {}", base, "bg-red-500/50 dark:bg-red-800/50") } else { format!("{} {}", base, "bg-white/50 dark:bg-gray-700/50") }
                                    }}>
                                        <div class="text-sm text-gray-700 dark:text-gray-300 mb-1">"Penalty"</div>
                                        <div class="text-xl font-bold text-gray-900 dark:text-gray-100">{move || format_pct(penalty_rate.get())}</div>
                                    </div>
                                </div>

                                <div class="flex items-center justify-between bg-white/50 dark:bg-gray-700/50 rounded-lg p-4 backdrop-blur-sm">
                                    <div>
                                        <div class="text-sm text-gray-700 dark:text-gray-300">"Current Edges:"</div>
                                        <div class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                                            {move || counter.get().to_string()}
                                            <span class="text-sm text-gray-600 dark:text-gray-400 ml-1">
                                                "/ " {move || minimum.get().to_string()}
                                            </span>
                                        </div>
                                    </div>
                                    <button
                                        node_ref=counter_btn_ref
                                        class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-lg shadow-md transform hover:scale-105 active:scale-95 transition-all duration-100 ease-in-out disabled:opacity-50 disabled:cursor-not-allowed"
                                        disabled=move || debt.get() <= 0.0 || is_won.get()
                                        on:click=on_click
                                    >
                                        "Edge!"
                                    </button>
                                </div>

                                <div class="relative flex justify-center w-full">
                                    <div
                                        class={move || {
                                            let base = "flex justify-center transition-all duration-300 ease-in-out";
                                            if win_enabled.get() && !is_won.get() { format!("{} {}", base, "opacity-100 transform translate-y-0") } else { format!("{} {}", base, "opacity-0 transform -translate-y-4 pointer-events-none") }
                                        }}
                                    >
                                        <button
                                            class="mt-2 bg-yellow-500 hover:bg-yellow-600 text-white font-bold py-3 px-6 rounded-lg shadow-lg transform hover:scale-105 transition-all duration-300 ease-in-out animate-pulse"
                                            on:click=on_roll
                                        >
                                            "Roll For a Chance To Cum!"
                                        </button>
                                    </div>

                                    <div
                                        class={move || {
                                            let base = "absolute top-0 left-0 w-full h-full flex items-center justify-center transition-all duration-500 ease-in-out pointer-events-none";
                                            if is_won.get() { format!("{} {}", base, "opacity-100") } else { format!("{} {}", base, "opacity-0") }
                                        }}
                                    >
                                        <div class="text-4xl font-bold text-green-600 dark:text-green-400 animate-bounce">
                                            "CUM!"
                                        </div>
                                    </div>
                                </div>


                                <div class="mt-4 text-center">
                                    <button
                                        class="text-sm text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors duration-200"
                                        on:click=on_reset
                                    >
                                        "Reset Game"
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </RandomBackgroundImage>
    }
}
