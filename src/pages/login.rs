use dioxus::prelude::*;

pub fn Login() -> Element {
    rsx! {
        div {
            class: "max-w-sm p-6 bg-white border border-gray-200 rounded-lg shadow-sm dark:bg-gray-800 dark:border-gray-700",
            h5 {
                class: "mb-2 text-2xl font-bold tracking-tight text-gray-900 dark:text-white",
                "Login"
            }
            form {
                div {
                    label {
                        r#for: "email",
                        class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                        "Email"
                    }
                    input {
                        r#type: "text",
                        id: "email",
                        class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        required: 1
                    }
                }
                div {
                    label {
                        r#for: "password",
                        class: "block mb-2 text-sm font-medium text-gray-900 dark:text-white",
                        "Password"
                    }
                    input {
                        r#type: "password",
                        id: "password",
                        class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                        required: 1
                    }
                }
            }
        }
    }
}
