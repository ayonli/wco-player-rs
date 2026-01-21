//! Series card component for displaying search results

use dioxus::prelude::*;
use wco::Series;

/// Series card component - displays a single series in a card format
#[component]
pub fn SeriesCard(
    /// Series data to display
    series: Series,
    /// Callback when card is clicked
    on_select: EventHandler<Series>,
) -> Element {
    let series_clone = series.clone();
    
    rsx! {
        div {
            class: "series-card",
            onclick: move |_| on_select.call(series_clone.clone()),
            
            div {
                class: "series-thumbnail",
                if let Some(ref thumb) = series.thumbnail {
                    img {
                        src: "{thumb}",
                        alt: "{series.title}",
                        loading: "lazy",
                    }
                } else {
                    div {
                        class: "no-thumbnail",
                        "🎬"
                    }
                }
            }
            
            div {
                class: "series-info",
                h3 {
                    class: "series-title",
                    "{series.title}"
                }
            }
        }
    }
}

/// Grid of series cards
#[component]
pub fn SeriesGrid(
    /// List of series to display
    series_list: Vec<Series>,
    /// Callback when a series is selected
    on_select: EventHandler<Series>,
) -> Element {
    rsx! {
        div {
            class: "series-grid",
            
            for series in series_list {
                SeriesCard {
                    key: "{series.url}",
                    series: series.clone(),
                    on_select: move |s| on_select.call(s),
                }
            }
        }
    }
}
