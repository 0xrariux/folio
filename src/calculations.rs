//! Bounded numeric expressions and relational aggregates. No scripts or executable code.
use crate::{
    fields::{Aggregate, CellValue, Column, ColumnKind},
    model::Page,
};
use std::collections::HashSet;

pub fn related<'a>(pages: &'a [Page], record: &Page, column: &Column) -> Vec<&'a Page> {
    if column.config.target_base.is_none() {
        return Vec::new();
    }
    if let Some(reverse) = &column.config.reverse_of {
        return pages.iter().filter(|p| !p.archived && p.parent_id == column.config.target_base && matches!(p.properties.custom.get(reverse), Some(CellValue::Relation(ids)) if ids.contains(&record.id))).collect();
    }
    let Some(CellValue::Relation(ids)) = record.properties.custom.get(&column.id) else {
        return Vec::new();
    };
    pages
        .iter()
        .filter(|p| !p.archived && p.parent_id == column.config.target_base && ids.contains(&p.id))
        .collect()
}
pub fn value(record: &Page, column: &Column, pages: &[Page]) -> Result<f64, String> {
    calculate(record, column, pages, &mut HashSet::new())
}
fn calculate(
    record: &Page,
    column: &Column,
    pages: &[Page],
    visited: &mut HashSet<(String, String)>,
) -> Result<f64, String> {
    let key = (record.id.clone(), column.id.clone());
    if visited.len() >= 32 || !visited.insert(key.clone()) {
        return Err("Circular reference".into());
    }
    let result = (|| match column.kind {
        ColumnKind::Number => Ok(record
            .properties
            .custom
            .get(&column.id)
            .and_then(CellValue::number)
            .unwrap_or(0.0)),
        ColumnKind::Checkbox => Ok(f64::from(matches!(
            record.properties.custom.get(&column.id),
            Some(CellValue::Checkbox(true))
        ))),
        ColumnKind::Relation => Ok(related(pages, record, column).len() as f64),
        ColumnKind::Rollup => {
            let base = pages
                .iter()
                .find(|p| Some(&p.id) == record.parent_id.as_ref())
                .and_then(|p| p.collection.as_ref())
                .ok_or("Database not found")?;
            let relation = base
                .columns
                .iter()
                .find(|c| {
                    Some(&c.id) == column.config.relation.as_ref() && c.kind == ColumnKind::Relation
                })
                .ok_or("Choose a relation")?;
            let records = related(pages, record, relation);
            let done = records
                .iter()
                .filter(|p| p.properties.status == crate::collection::Status::Done)
                .count() as f64;
            let count = records.len() as f64;
            match column.config.aggregate {
                Aggregate::Count => Ok(count),
                Aggregate::Done => Ok(done),
                Aggregate::Remaining => Ok(count - done),
                Aggregate::Progress => Ok(if count == 0.0 {
                    0.0
                } else {
                    done * 100.0 / count
                }),
                Aggregate::Sum => {
                    let target = pages
                        .iter()
                        .find(|p| Some(&p.id) == relation.config.target_base.as_ref())
                        .and_then(|p| p.collection.as_ref())
                        .and_then(|c| {
                            c.columns
                                .iter()
                                .find(|c| Some(&c.id) == column.config.property.as_ref())
                        })
                        .ok_or("Choose a numeric property")?;
                    records.into_iter().try_fold(0.0, |sum, p| {
                        calculate(p, target, pages, visited).map(|v| sum + v)
                    })
                }
            }
        }
        ColumnKind::Formula => {
            let base = pages
                .iter()
                .find(|p| Some(&p.id) == record.parent_id.as_ref())
                .and_then(|p| p.collection.as_ref())
                .ok_or("Database not found")?;
            let mut expression = column.config.expression.clone();
            if expression.len() > 2000 {
                return Err("Formula too long".into());
            }
            while let Some(start) = expression.find("prop(\"") {
                let rest = &expression[start + 6..];
                let end = rest.find("\")").ok_or("Use prop(\"Name\")")?;
                let name = &rest[..end];
                let field = base
                    .columns
                    .iter()
                    .find(|c| c.name == name || c.id == name)
                    .ok_or_else(|| format!("Unknown property: {name}"))?;
                let n = calculate(record, field, pages, visited)?;
                expression.replace_range(start..start + 6 + end + 2, &format!("({n})"));
            }
            expression = expression.replace(
                "status_done",
                if record.properties.status == crate::collection::Status::Done {
                    "1"
                } else {
                    "0"
                },
            );
            evaluate(&expression)
        }
        _ => Err("Property is not numeric".into()),
    })();
    visited.remove(&key);
    result.and_then(|n| {
        if n.is_finite() {
            Ok(n)
        } else {
            Err("Result out of range".into())
        }
    })
}
pub fn display(record: &Page, column: &Column, pages: &[Page]) -> String {
    match column.kind {
        ColumnKind::Relation => {
            let names: Vec<_> = related(pages, record, column)
                .into_iter()
                .map(Page::display_title)
                .collect();
            if names.is_empty() {
                "—".into()
            } else {
                names.join(", ")
            }
        }
        ColumnKind::Rollup | ColumnKind::Formula => value(record, column, pages)
            .map(crate::fields::format_number)
            .unwrap_or_else(|e| format!("⚠ {e}")),
        _ => record
            .properties
            .custom
            .get(&column.id)
            .map(CellValue::display)
            .unwrap_or_else(|| "—".into()),
    }
}

pub fn evaluate(expression: &str) -> Result<f64, String> {
    struct Parser<'a> {
        text: &'a [u8],
        at: usize,
        depth: usize,
    }
    impl Parser<'_> {
        fn space(&mut self) {
            while self.text.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
        }
        fn take(&mut self, c: u8) -> bool {
            self.space();
            if self.text.get(self.at) == Some(&c) {
                self.at += 1;
                true
            } else {
                false
            }
        }
        fn sum(&mut self) -> Result<f64, String> {
            let mut n = self.product()?;
            loop {
                if self.take(b'+') {
                    n += self.product()?;
                } else if self.take(b'-') {
                    n -= self.product()?;
                } else {
                    return Ok(n);
                }
            }
        }
        fn product(&mut self) -> Result<f64, String> {
            let mut n = self.atom()?;
            loop {
                if self.take(b'*') {
                    n *= self.atom()?;
                } else if self.take(b'/') {
                    let d = self.atom()?;
                    if d == 0.0 {
                        return Err("Division by zero".into());
                    }
                    n /= d;
                } else {
                    return Ok(n);
                }
            }
        }
        fn atom(&mut self) -> Result<f64, String> {
            self.depth += 1;
            if self.depth > 32 {
                return Err("Formula nested too deeply".into());
            }
            let result = if self.take(b'-') {
                self.atom().map(|n| -n)
            } else if self.take(b'+') {
                self.atom()
            } else if self.take(b'(') {
                let n = self.sum()?;
                if !self.take(b')') {
                    return Err("Missing parenthesis".into());
                }
                Ok(n)
            } else {
                self.space();
                let start = self.at;
                while self
                    .text
                    .get(self.at)
                    .is_some_and(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E'))
                {
                    self.at += 1;
                }
                std::str::from_utf8(&self.text[start..self.at])
                    .ok()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or("Expected a number or property".into())
            };
            self.depth -= 1;
            result
        }
    }
    if expression.len() > 2000 {
        return Err("Formula too long".into());
    }
    let mut parser = Parser {
        text: expression.as_bytes(),
        at: 0,
        depth: 0,
    };
    let n = parser.sum()?;
    parser.space();
    if parser.at != parser.text.len() || !n.is_finite() {
        return Err("Invalid formula or result out of range".into());
    }
    Ok(n)
}
