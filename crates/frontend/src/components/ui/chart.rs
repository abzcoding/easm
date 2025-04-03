use leptos::prelude::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn btoa(s: &str) -> String;
}

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
    // Encode chart data to a data URL that could be used with a JS charting library
    let chart_config = format!(
        r#"{{
            "type": "{}",
            "data": {{
                "labels": [{}],
                "datasets": [{}]
            }},
            "options": {{
                "responsive": true,
                "plugins": {{
                    "legend": {{
                        "display": {}
                    }},
                    "title": {{
                        "display": true,
                        "text": "{}"
                    }}
                }}
            }}
        }}"#,
        chart_type,
        data.labels
            .iter()
            .map(|l| format!("\"{}\"", l))
            .collect::<Vec<_>>()
            .join(","),
        data.datasets
            .iter()
            .map(|d| {
                let bg_colors = match &d.background_colors {
                    Some(colors) => format!(
                        "[{}]",
                        colors
                            .iter()
                            .map(|c| format!("\"{}\"", c))
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    None => "\"rgba(75, 192, 192, 0.2)\"".to_string(),
                };
                let border_colors = match &d.border_colors {
                    Some(colors) => format!(
                        "[{}]",
                        colors
                            .iter()
                            .map(|c| format!("\"{}\"", c))
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    None => "\"rgba(75, 192, 192, 1)\"".to_string(),
                };
                format!(
                    r#"{{
                        "label": "{}",
                        "data": [{}],
                        "backgroundColor": {},
                        "borderColor": {},
                        "borderWidth": 1
                    }}"#,
                    d.label,
                    d.data
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    bg_colors,
                    border_colors
                )
            })
            .collect::<Vec<_>>()
            .join(","),
        show_legend,
        title
    );

    // Construct a URL for the chart (in a real implementation, we'd render with a JS library)
    let encoded_config = btoa(&chart_config);
    let chart_url = format!("https://quickchart.io/chart?c={}", encoded_config);

    view! {
        <div class="chart-container" style={format!("width: {}; height: {}px;", width, height)}>
            <img
                src={chart_url}
                alt={format!("{} chart", title)}
                class="chart-image"
                width={width.clone()}
                height={height.to_string()}
            />
        </div>
    }
}
