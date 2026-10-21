use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub label: String,
    pub value: f64,
}

pub struct ChartGenerator;

impl ChartGenerator {
    /// Generates a clean standalone SVG Bar Chart
    pub fn generate_bar_chart_svg(title: &str, series: &[ChartSeries], width: usize, height: usize) -> String {
        let max_val = series
            .iter()
            .map(|s| s.value)
            .fold(0.0, f64::max)
            .max(1.0);

        let margin_left = 80;
        let margin_bottom = 60;
        let margin_top = 40;
        let margin_right = 40;

        let plot_width = width.saturating_sub(margin_left + margin_right);
        let plot_height = height.saturating_sub(margin_top + margin_bottom);

        let bar_count = series.len().max(1);
        let bar_width = (plot_width / bar_count).saturating_sub(15).max(10);
        let bar_spacing = plot_width / bar_count;

        let mut svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="background-color: #1e1e2e; font-family: sans-serif;">
  <text x="{}" y="25" fill="#cdd6f4" font-size="16" font-weight="bold" text-anchor="middle">{}</text>
  <!-- Axes -->
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6c7086" stroke-width="2"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6c7086" stroke-width="2"/>
"##,
            width, height, width, height, width / 2, title,
            margin_left, margin_top, margin_left, height - margin_bottom,
            margin_left, height - margin_bottom, width - margin_right, height - margin_bottom
        );

        let colors = ["#89b4fa", "#a6e3a1", "#f9e2af", "#fab387", "#f38ba8", "#cba6f7"];

        for (i, item) in series.iter().enumerate() {
            let bar_h = ((item.value / max_val) * (plot_height as f64)) as usize;
            let x = margin_left + i * bar_spacing + 10;
            let y = (height - margin_bottom).saturating_sub(bar_h);
            let color = colors[i % colors.len()];

            svg.push_str(&format!(
                r##"  <rect x="{}" y="{}" width="{}" height="{}" fill="{}" rx="4">
    <title>{}: {}</title>
  </rect>
  <text x="{}" y="{}" fill="#cdd6f4" font-size="11" text-anchor="middle">{}</text>
  <text x="{}" y="{}" fill="#a6adc8" font-size="10" text-anchor="middle" transform="rotate(-25 {} {})">{}</text>
"##,
                x, y, bar_width, bar_h, color,
                item.label, item.value,
                x + bar_width / 2, y.saturating_sub(5), item.value,
                x + bar_width / 2, height - margin_bottom + 20, x + bar_width / 2, height - margin_bottom + 20, item.label
            ));
        }

        svg.push_str("</svg>");
        svg
    }

    /// Generates a clean standalone SVG Line Chart
    pub fn generate_line_chart_svg(title: &str, series: &[ChartSeries], width: usize, height: usize) -> String {
        let max_val = series
            .iter()
            .map(|s| s.value)
            .fold(0.0, f64::max)
            .max(1.0);

        let margin_left = 60;
        let margin_bottom = 50;
        let margin_top = 40;
        let margin_right = 40;

        let plot_width = width.saturating_sub(margin_left + margin_right);
        let plot_height = height.saturating_sub(margin_top + margin_bottom);

        let count = series.len().max(2);
        let step_x = plot_width as f64 / (count - 1) as f64;

        let mut points_str = String::new();
        let mut circles = String::new();

        for (i, item) in series.iter().enumerate() {
            let x = margin_left as f64 + i as f64 * step_x;
            let norm_y = item.value / max_val;
            let y = (height - margin_bottom) as f64 - (norm_y * plot_height as f64);

            points_str.push_str(&format!("{:.1},{:.1} ", x, y));
            circles.push_str(&format!(
                r##"  <circle cx="{:.1}" cy="{:.1}" r="4" fill="#f38ba8"/>
  <text x="{:.1}" y="{:.1}" fill="#cdd6f4" font-size="10" text-anchor="middle">{:.1}</text>
"##,
                x, y, x, y - 8.0, item.value
            ));
        }

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}" style="background-color: #1e1e2e; font-family: sans-serif;">
  <text x="{}" y="25" fill="#cdd6f4" font-size="16" font-weight="bold" text-anchor="middle">{}</text>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6c7086" stroke-width="2"/>
  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="#6c7086" stroke-width="2"/>
  <polyline fill="none" stroke="#89b4fa" stroke-width="3" points="{}"/>
{}</svg>"##,
            width, height, width, height, width / 2, title,
            margin_left, margin_top, margin_left, height - margin_bottom,
            margin_left, height - margin_bottom, width - margin_right, height - margin_bottom,
            points_str.trim(),
            circles
        )
    }
}
