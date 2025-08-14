use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum Size {
    Small,
    Medium,
    _Large,
}

const COLORS: [&str; 8] = [
    "bg-red-500",
    "bg-blue-500",
    "bg-green-500",
    "bg-yellow-500",
    "bg-purple-500",
    "bg-pink-500",
    "bg-indigo-500",
    "bg-gray-500",
];

#[component]
pub fn Avatar(src: Option<String>, alt: String, size: Size) -> Element {
    let size_class = match size {
        Size::Small => "w-8 h-8 text-sm",
        Size::Medium => "w-10 h-10 text-base",
        Size::_Large => "w-12 h-12 text-lg",
    };

    if let Some(src) = src {
        if !src.is_empty() {
            return rsx! {
                img {
                    src,
                    alt,
                    class: "{size_class} rounded-full flex items-center justify-center font-semibold text-white"
                }
            };
        }
    }

    let initials: String = alt
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase()
        .chars()
        .take(2)
        .collect();

    let color_idx = initials
        .chars()
        .fold(0, |pv, cv| (pv + cv as usize) % COLORS.len());

    rsx! {
        div {
            class: "{COLORS[color_idx]} rounded-full flex items-center justify-center font-semibold text-white {size_class}",
            "{initials}"
        }
    }
}
