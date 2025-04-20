//! Controller display utilities

use alloc::{
    rc::Rc,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{cell::RefCell, mem, time::Duration};

use futures::FutureExt;
use vexide::{prelude::Controller, sync::Mutex, time::sleep};

const SAFE_UPDATE_DURATION: Duration = Duration::from_millis(200);

pub const fn controller_char_width(letter: char) -> usize {
    match letter {
        'a' => 6,
        'b' => 6,
        'c' => 5,
        'd' => 6,
        'e' => 6,
        'f' => 4,
        'g' => 6,
        'h' => 6,
        'i' => 1,
        'j' => 3,
        'k' => 5,
        'l' => 1,
        'm' => 9,
        'n' => 6,
        'o' => 6,
        'p' => 6,
        'q' => 6,
        'r' => 4,
        's' => 6,
        't' => 5,
        'u' => 6,
        'v' => 6,
        'w' => 10,
        'x' => 5,
        'y' => 6,
        'z' => 5,
        'A' => 8,
        'B' => 6,
        'C' => 7,
        'D' => 7,
        'E' => 5,
        'F' => 5,
        'G' => 8,
        'H' => 7,
        'I' => 3,
        'J' => 3,
        'K' => 6,
        'L' => 5,
        'M' => 9,
        'N' => 7,
        'O' => 8,
        'P' => 6,
        'Q' => 8,
        'R' => 6,
        'S' => 6,
        'T' => 7,
        'U' => 7,
        'V' => 8,
        'W' => 12,
        'X' => 7,
        'Y' => 7,
        'Z' => 6,
        '0' => 6,
        '1' => 3,
        '2' => 6,
        '3' => 6,
        '4' => 7,
        '5' => 6,
        '6' => 6,
        '7' => 6,
        '8' => 6,
        '9' => 6,
        ' ' => 4,
        '!' => 1,
        '"' => 3,
        '#' => 8,
        '$' => 6,
        '%' => 10,
        '&' => 8,
        '\'' => 1,
        '(' => 2,
        ')' => 2,
        '*' => 6,
        '+' => 7,
        ',' => 1,
        '-' => 3,
        '.' => 1,
        '/' => 5,
        ':' => 1,
        ';' => 1,
        '<' => 6,
        '=' => 6,
        '>' => 6,
        '?' => 5,
        '@' => 10,
        '[' => 3,
        '\\' => 5,
        ']' => 3,
        '^' => 6,
        '_' => 6,
        '`' => 2,
        '{' => 4,
        '|' => 1,
        '}' => 4,
        '~' => 6,
        _ => panic!("Invalid letter"),
    }
}

pub fn controller_str_width(s: &str) -> usize {
    s.chars().map(controller_char_width).sum::<usize>() + s.len().saturating_sub(1)
}

const CONTROLLER_WIDTH: usize = 128;

/// Returns a string filled with the provided underlining character
pub fn underline_string(s: &str, c: char, range: core::ops::Range<usize>) -> String {
    // Find which pixels have the underlined words
    let underlined_pixels: core::ops::Range<usize> =
        controller_str_width(&s[0..range.start])..controller_str_width(&s[0..range.end]);

    // Create a new string with the underlined characters
    let mut result = String::new();
    let mut current_width = 0;
    let char_width = controller_char_width(c);
    while current_width < CONTROLLER_WIDTH {
        if (underlined_pixels.start..(underlined_pixels.end - char_width / 2))
            .contains(&current_width)
        {
            result.push(c);
            current_width += char_width + 1;
        } else {
            result.push(' ');
            current_width += controller_char_width(' ') + 1;
        }
    }

    result
}

pub fn center_string(s: &str) -> String {
    const PADDING_CHAR: char = ' ';
    const PADDING_WIDTH: usize = controller_char_width(PADDING_CHAR) + 1;
    let width = controller_str_width(s);
    let padding = ((CONTROLLER_WIDTH - width) / 2) / PADDING_WIDTH;
    let mut result = String::new();
    result.push_str(&PADDING_CHAR.to_string().repeat(padding));
    result.push_str(s);
    result.push_str(&PADDING_CHAR.to_string().repeat(padding));
    result
}

/// Prompts the user to select
#[allow(clippy::await_holding_refcell_ref)]
pub async fn horizontal_picker(
    controller: Arc<Mutex<Controller>>,
    prompt: &str,
    options: Vec<String>,
) -> Option<usize> {
    const ITEM_SEPARATOR: &str = "   ";
    const ITEM_UNDERLINE: char = '^';

    let selected = Rc::new(RefCell::new(0));
    {
        let mut controller: vexide::sync::MutexGuard<Controller> = controller.lock().await;
        controller.screen.clear_screen().await.unwrap();
        sleep(Duration::from_millis(100)).await;
        controller
            .screen
            .try_set_text(center_string(prompt), 1, 1)
            .unwrap();
    }
    futures::select_biased!(
        x = (async {
            let selected = selected.clone();
            loop {
                {
                    let mut controller: vexide::sync::MutexGuard<Controller> = controller.lock().await;

                    // Handle button presses
                    let mut selected = selected.borrow_mut();
                    let state = controller.state().unwrap_or_default();

                    if state.button_left.is_now_pressed() {
                        *selected = (*selected + options.len() - 1) % options.len();
                    } else if state.button_right.is_now_pressed() {
                        *selected = (*selected + 1) % options.len();
                    } else if state.button_a.is_now_pressed() {
                        let selected_ref = *selected;
                        mem::drop(selected);
                        sleep(SAFE_UPDATE_DURATION).await;
                        controller.screen.clear_screen().await.unwrap();
                        break Some(selected_ref);
                    } else if state.button_b.is_now_pressed() {
                        mem::drop(selected);
                        sleep(SAFE_UPDATE_DURATION).await;
                        controller.screen.clear_screen().await.unwrap();
                        break None;
                    }
                }
                sleep(Duration::from_millis(20)).await;
            }
        }).fuse() => { x },
        () = (async {
            let selected = selected.clone();
            loop {
                {
                    let mut controller_ref: vexide::sync::MutexGuard<Controller> = controller.lock().await;

                    // Render to the screen
                    let selected = { *selected.borrow() };
                    let offset: usize =
                        options[0..selected]
                            .iter()
                            .map(|s| s.len() + ITEM_SEPARATOR.len())
                            .sum::<usize>()
                            .saturating_sub(5);
                    let mut list = options[0..].join(ITEM_SEPARATOR);
                    list.replace_range(0..offset, "");
                    let mut at_end = false;
                    while controller_str_width(&list) < CONTROLLER_WIDTH {
                        list.push(' ');
                        at_end = true;
                    }
                    while controller_str_width(&list) > CONTROLLER_WIDTH {
                        list.pop();
                    }
                    if !at_end {
                        list.pop();
                        list.pop();
                        list.push_str("> ");
                    }
                    _ = controller_ref.screen.try_set_text(&list, 2, 1);
                    mem::drop(controller_ref);
                    sleep(SAFE_UPDATE_DURATION).await;
                    let mut controller_ref: vexide::sync::MutexGuard<Controller> = controller.lock().await;
                    let range_start = options[0..selected]
                        .iter()
                        .map(|s| s.len() + ITEM_SEPARATOR.len())
                        .sum::<usize>() - offset;
                    let range = range_start..(range_start + options[selected].len());
                    let underline = underline_string(&list, ITEM_UNDERLINE, range);
                    _ = controller_ref.screen.try_set_text(underline, 3, 1);
                }
                sleep(SAFE_UPDATE_DURATION).await;
            }
        }).fuse() => { None },
    )
}

pub async fn simple_dialog(controller: Arc<Mutex<Controller>>, title: &str, description: &str) {
    let mut controller: vexide::sync::MutexGuard<Controller> = controller.lock().await;
    sleep(SAFE_UPDATE_DURATION).await;
    _ = controller.screen.try_set_text(center_string(title), 1, 1);
    sleep(SAFE_UPDATE_DURATION).await;
    _ = controller
        .screen
        .try_set_text(center_string(description), 2, 2);
}
