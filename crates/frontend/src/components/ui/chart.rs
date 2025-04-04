use leptos::prelude::*;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Doughnut,
}

impl Display for ChartType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ChartType::Bar => write!(f, "bar"),
            ChartType::Line => write!(f, "line"),
            ChartType::Pie => write!(f, "pie"),
            ChartType::Doughnut => write!(f, "doughnut"),
        }
    }
}

#[derive(Clone)]
pub struct ChartData {
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
}

#[derive(Clone)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
    pub background_colors: Option<Vec<String>>,
    pub border_colors: Option<Vec<String>>,
}

#[component]
pub fn Chart(
    #[prop(into)] title: String,
    #[prop(into)] chart_type: ChartType,
    #[prop(into)] data: ChartData,
    #[prop(default = 300)] height: u32,
    #[prop(default = "100%".to_string())] width: String,
    #[prop(default = false)] show_legend: bool,
) -> impl IntoView {
    // Extremely simple chart implementation
    let style = format!("width: {}; height: {}px; background-color: #f8f9fa; border-radius: 8px; padding: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);", width, height);
    let chart_type_str = chart_type.to_string();

    // Format data
    let formatted_data = data
        .labels
        .iter()
        .enumerate()
        .filter_map(|(i, label)| {
            if i < data.datasets[0].data.len() {
                Some(format!("{}: {}", label, data.datasets[0].data[i]))
            } else {
                None
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    view! {
        <div class="chart-container" style={style}>
            <h3 class="chart-title">{title}</h3>
            <p class="chart-type">"Type: " {chart_type_str}</p>
            <p class="chart-data">"Data: " {formatted_data}</p>

            // Show legend unconditionally but don't display content if show_legend is false
            <p class="chart-legend" style={if show_legend { "" } else { "display: none;" }}>
                "Legend: " {data.datasets[0].label.clone()}
            </p>
        </div>
    }
}
