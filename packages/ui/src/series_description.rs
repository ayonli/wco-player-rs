//! Series description component for displaying series details

use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::{LdChevronDown, LdChevronRight}};
use wco::SeriesDetail;

/// Series description component
#[component]
pub fn SeriesDescription(
    /// Series detail data
    detail: SeriesDetail,
    /// Whether description is expanded
    description_expanded: Signal<bool>,
    /// Callback when expand/collapse is toggled
    on_toggle: EventHandler<()>,
) -> Element {
    // Only render if there's content to show
    if detail.description.is_none() && detail.thumbnail.is_none() && detail.tags.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "series-description-section",
            div {
                class: "series-description-header",
                onclick: move |_| on_toggle.call(()),
                h3 { class: "series-description-title", "Description" }
                button { class: "description-toggle",
                    if description_expanded() {
                        Icon {
                            icon: LdChevronDown,
                            width: Some(20),
                            height: Some(20),
                        }
                    } else {
                        Icon {
                            icon: LdChevronRight,
                            width: Some(20),
                            height: Some(20),
                        }
                    }
                }
            }
            div { class: if description_expanded() { "series-description-content expanded" } else { "series-description-content collapsed" },
                div { class: "series-description-body",
                    if let Some(thumbnail) = &detail.thumbnail {
                        div { class: "series-thumbnail-wrapper",
                            img {
                                class: "series-thumbnail-img",
                                src: "{thumbnail}",
                                alt: "Series thumbnail",
                            }
                        }
                    }
                    if let Some(description) = &detail.description {
                        div { class: "series-description-text", "{description}" }
                    }
                    if !detail.tags.is_empty() {
                        div { class: "series-tags",
                            div { class: "series-tags-header", "Tags" }
                            div { class: "series-tags-list",
                                for tag in &detail.tags {
                                    span { class: "series-tag", "{tag}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
