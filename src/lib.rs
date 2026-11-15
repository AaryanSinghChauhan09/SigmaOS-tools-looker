#![allow(non_snake_case)]
//! Looker LookML Dimension & Measure Resolver for SigmaOS

#[derive(Debug, Clone, PartialEq)]
pub enum AggregationType {
    Count,
    Sum,
    Average,
}

#[derive(Debug, Clone)]
pub enum LookMlField {
    Dimension {
        name: String,
        column: String,
    },
    Measure {
        name: String,
        agg: AggregationType,
        column: String,
    },
}

pub struct LookMlView {
    pub name: String,
    pub sql_table_name: String,
    pub fields: Vec<LookMlField>,
}

impl LookMlView {
    pub fn new(name: &str, table: &str, fields: Vec<LookMlField>) -> Self {
        Self {
            name: name.to_string(),
            sql_table_name: table.to_string(),
            fields,
        }
    }

    pub fn generate_sql(&self, selected_fields: &[&str]) -> Result<String, String> {
        let mut select_clauses = Vec::new();
        let mut group_by_indices = Vec::new();

        for (idx, &fname) in selected_fields.iter().enumerate() {
            let field = self
                .fields
                .iter()
                .find(|f| match f {
                    LookMlField::Dimension { name, .. } => name == fname,
                    LookMlField::Measure { name, .. } => name == fname,
                })
                .ok_or_else(|| format!("Field '{}' not defined in LookML view", fname))?;

            match field {
                LookMlField::Dimension { column, .. } => {
                    select_clauses.push(format!("{} AS {}", column, fname));
                    group_by_indices.push(idx + 1);
                }
                LookMlField::Measure { agg, column, .. } => {
                    let agg_str = match agg {
                        AggregationType::Count => format!("COUNT({})", column),
                        AggregationType::Sum => format!("SUM({})", column),
                        AggregationType::Average => format!("AVG({})", column),
                    };
                    select_clauses.push(format!("{} AS {}", agg_str, fname));
                }
            }
        }

        let mut query = format!(
            "SELECT {} FROM {}",
            select_clauses.join(", "),
            self.sql_table_name
        );
        if !group_by_indices.is_empty() && select_clauses.iter().any(|c| c.contains('(')) {
            let groups: Vec<String> = group_by_indices
                .into_iter()
                .map(|i| i.to_string())
                .collect();
            query.push_str(&format!(" GROUP BY {}", groups.join(", ")));
        }

        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookml_sql_generation_with_group_by_aaa() {
        // Arrange
        let view = LookMlView::new(
            "orders",
            "warehouse.orders",
            vec![
                LookMlField::Dimension {
                    name: "status".into(),
                    column: "order_status".into(),
                },
                LookMlField::Measure {
                    name: "total_revenue".into(),
                    agg: AggregationType::Sum,
                    column: "amount".into(),
                },
            ],
        );

        // Act
        let sql = view
            .generate_sql(&["status", "total_revenue"])
            .expect("SQL generation failed");

        // Assert
        assert_eq!(sql, "SELECT order_status AS status, SUM(amount) AS total_revenue FROM warehouse.orders GROUP BY 1");
    }
}
