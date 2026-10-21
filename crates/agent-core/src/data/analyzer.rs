use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::spreadsheet::SpreadsheetTable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSummary {
    pub column_name: String,
    pub count: usize,
    pub null_count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupResult {
    pub key: String,
    pub count: usize,
    pub aggregate_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierResult {
    pub row_index: usize,
    pub value: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

pub struct DataAnalyzer;

impl DataAnalyzer {
    pub fn summarize_column(table: &SpreadsheetTable, column_name: &str) -> Result<ColumnSummary, String> {
        let col_idx = table
            .headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| format!("Column '{}' not found in table", column_name))?;

        let mut values: Vec<f64> = Vec::new();
        let mut null_count = 0;

        for row in &table.rows {
            if let Some(val) = row.get(col_idx) {
                if let Ok(num) = val.trim().parse::<f64>() {
                    values.push(num);
                } else {
                    null_count += 1;
                }
            } else {
                null_count += 1;
            }
        }

        if values.is_empty() {
            return Err(format!("No numeric values found in column '{}'", column_name));
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let count = values.len();
        let sum: f64 = values.iter().sum();
        let mean = sum / (count as f64);

        let median = if count % 2 == 0 {
            (values[count / 2 - 1] + values[count / 2]) / 2.0
        } else {
            values[count / 2]
        };

        let min = *values.first().unwrap();
        let max = *values.last().unwrap();

        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (count as f64);
        let std_dev = variance.sqrt();

        Ok(ColumnSummary {
            column_name: column_name.to_string(),
            count,
            null_count,
            sum,
            mean,
            median,
            min,
            max,
            std_dev,
        })
    }

    pub fn group_by(
        table: &SpreadsheetTable,
        group_col: &str,
        agg_col: &str,
        agg_op: &str, // "sum", "mean", "count", "min", "max"
    ) -> Result<Vec<GroupResult>, String> {
        let g_idx = table
            .headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(group_col))
            .ok_or_else(|| format!("Group column '{}' not found", group_col))?;
        let a_idx = table
            .headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(agg_col))
            .ok_or_else(|| format!("Aggregate column '{}' not found", agg_col))?;

        let mut groups: HashMap<String, Vec<f64>> = HashMap::new();

        for row in &table.rows {
            let key = row.get(g_idx).map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown".to_string());
            if let Some(val_str) = row.get(a_idx) {
                if let Ok(num) = val_str.trim().parse::<f64>() {
                    groups.entry(key).or_default().push(num);
                }
            }
        }

        let mut results = Vec::new();
        for (key, vals) in groups {
            let count = vals.len();
            let agg_val = match agg_op.to_lowercase().as_str() {
                "sum" => vals.iter().sum::<f64>(),
                "mean" | "avg" => {
                    if count > 0 {
                        vals.iter().sum::<f64>() / (count as f64)
                    } else {
                        0.0
                    }
                }
                "min" => vals.iter().cloned().fold(f64::INFINITY, f64::min),
                "max" => vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                "count" => count as f64,
                _ => vals.iter().sum::<f64>(),
            };

            results.push(GroupResult {
                key,
                count,
                aggregate_value: agg_val,
            });
        }

        results.sort_by(|a, b| b.aggregate_value.partial_cmp(&a.aggregate_value).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    pub fn detect_outliers_iqr(table: &SpreadsheetTable, column_name: &str) -> Result<Vec<OutlierResult>, String> {
        let col_idx = table
            .headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let mut indexed_vals: Vec<(usize, f64)> = Vec::new();
        for (i, row) in table.rows.iter().enumerate() {
            if let Some(val_str) = row.get(col_idx) {
                if let Ok(num) = val_str.trim().parse::<f64>() {
                    indexed_vals.push((i, num));
                }
            }
        }

        if indexed_vals.len() < 4 {
            return Ok(Vec::new()); // Not enough data for meaningful IQR
        }

        indexed_vals.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let n = indexed_vals.len();
        let q1 = indexed_vals[n / 4].1;
        let q3 = indexed_vals[(3 * n) / 4].1;
        let iqr = q3 - q1;
        let lower_bound = q1 - 1.5 * iqr;
        let upper_bound = q3 + 1.5 * iqr;

        let mut outliers = Vec::new();
        for (orig_idx, val) in indexed_vals {
            if val < lower_bound || val > upper_bound {
                outliers.push(OutlierResult {
                    row_index: orig_idx,
                    value: val,
                    lower_bound,
                    upper_bound,
                });
            }
        }

        Ok(outliers)
    }

    pub fn filter(
        table: &SpreadsheetTable,
        column_name: &str,
        predicate: impl Fn(&str) -> bool,
    ) -> Result<SpreadsheetTable, String> {
        let col_idx = table
            .headers
            .iter()
            .position(|h| h.eq_ignore_ascii_case(column_name))
            .ok_or_else(|| format!("Column '{}' not found", column_name))?;

        let filtered_rows: Vec<Vec<String>> = table
            .rows
            .iter()
            .filter(|row| {
                if let Some(val) = row.get(col_idx) {
                    predicate(val)
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let total_rows = filtered_rows.len();
        Ok(SpreadsheetTable {
            name: format!("{}_filtered", table.name),
            headers: table.headers.clone(),
            columns: table.columns.clone(),
            rows: filtered_rows,
            total_rows,
        })
    }
}
