use std::fs;
use std::path::Path;

use config::Config;
use entities::project::ProjectTuple;
use genpdf::elements::{Break, Image, LinearLayout, PageBreak, Paragraph};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::{elements, style};
use genpdf::{Alignment, Element, Scale};
use lazy_static::lazy_static;
use regex::Regex;
use std::time::Duration;



const IMAGE_PATH_JPG: &str = "server/static/images/logo.jpg";

pub fn generate_pdf(
    config: &Config,
    project_list: Vec<ProjectTuple>,
    target_file_name: &String,
    language: &str,
) -> Result<(), genpdf::error::Error> {
    let font = get_font(config);
    let lines_per_page = get_lines_per_page(config);
    let title = get_title(language);
    let mut post_address_lines: Vec<String> = Vec::new();
    let mut mailphone_lines: Vec<String> = Vec::new();

    let logo_scale = match config.get_float("pdf_logo_scale") {
        Ok(val) => Some(val),
        _ => None,
    };

    get_env_vars_by_prefix(config, "address_line_", language, &mut post_address_lines);
    get_env_vars_by_prefix(
        config,
        "web_and_phone_line_",
        language,
        &mut mailphone_lines,
    );

    let font_family_roboto = genpdf::fonts::from_files("./pdf/fonts", font.as_str(), None)
        .expect("Failed to load font family");

    let document = generate_document(
        font_family_roboto,
        title.to_string(),
        project_list,
        lines_per_page,
        language,
        post_address_lines,
        mailphone_lines,
        logo_scale,
    )?;

   delete_file_with_delay(target_file_name.clone(), 30);


    document.render_to_file(target_file_name)
}

fn generate_document(
    font_family: FontFamily<FontData>,
    title: String,
    project_list: Vec<ProjectTuple>,
    lines_per_page: i64,
    language: &str,
    post_address_lines: Vec<String>,
    mailphone_lines: Vec<String>,
    logo_scale: Option<f64>,
) -> Result<genpdf::Document, genpdf::error::Error> {
    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(&title);
    doc.set_page_decorator(add_header(
        post_address_lines,
        mailphone_lines,
        logo_scale,
        7,
    ));
    doc.push(
        elements::Paragraph::new(title)
            .aligned(Alignment::Center)
            .styled(style::Style::new().bold().with_font_size(16)),
    );
    doc.push(elements::Break::new(1.5));

    doc = render_projects(doc, project_list, lines_per_page, language);

    Ok(doc)
}

fn render_projects(
    mut doc: genpdf::Document,
    project_tuples: Vec<ProjectTuple>,
    lines_per_page: i64,
    language: &str,
) -> genpdf::Document {
    let mut current_page_lines: i64 = 0;
    let mut first_page = true;

    for project_tuple in project_tuples {
        let project = project_tuple.0;
        let clients = project_tuple.1.first();
        let businessareas = project_tuple.2.first();
        let roles = project_tuple.3;
        let person = project_tuple.4.first();

        let technologies = project_tuple.5;

        let roles_string = if !roles.is_empty() {
            let roles_as_strings: Vec<String> = roles
                .iter()
                .map(|r| -> String {
                    match language {
                        "de" => r.name_de.clone(),
                        _ => r.name_en.clone(),
                    }
                })
                .collect();

            match language {
                "de" => format!("als {}", roles_as_strings.join(", ")),
                _ => format!("as {}", roles_as_strings.join(", ")),
            }
        } else {
            String::from("")
        };

        let tech_string = if !technologies.is_empty() {
            let technologies_as_strings: Vec<String> = technologies
                .iter()
                .map(|t| -> String { t.name.clone() })
                .collect();
            match language {
                "de" => format!("Technologien: {}", technologies_as_strings.join(", ")),
                _ => format!("Technologies: {}", technologies_as_strings.join(", ")),
            }
        } else {
            String::from("")
        };

        let client_name = match clients {
            Some(client) => client.name.to_owned(),
            None => "[INVALD_DATA]".to_string(),
        };
        // _businessarea_name not yet used
        let _businessarea_name = match businessareas {
            Some(businessarea) => match language {
                "de" => businessarea.name_de.to_owned(),
                _ => businessarea.name_en.to_owned(),
            },
            None => "[INVALD_DATA]".to_string(),
        };

        let _person_name = match person {
            Some(p) => p.name.to_owned(),
            None => "[INVALD_DATA]".to_string(),
        };
        let description = match language {
            "de" => project.description_de,
            _ => project.description_en,
        };
        let summary = match language {
            "de" => project.summary_de,
            _ => project.summary_en,
        };

        let description_iterator = description.split('\n');

        let current_table_line_coount = i64::try_from(description.matches('\n').count()).unwrap()
            + i64::try_from(summary.matches('\n').count()).unwrap();
        current_page_lines += current_table_line_coount;
        let max_for_this_page = match first_page {
            true => lines_per_page - 3, // due to title on first page
            false => lines_per_page,
        };

        let force_page_break = if current_page_lines > max_for_this_page {
            // move current table to new page and reset line count to current's table line count
            log::info!(
                "current_page_lines {}, lines_per_page {}",
                current_page_lines,
                lines_per_page
            );
            current_page_lines = current_table_line_coount;

            if first_page {
                first_page = false;
            }
            true
        } else {
            false
        };

        let style_small = style::Style::new().with_font_size(10);
        let mut style_small_grey = style_small;
        style_small_grey.set_color(style::Color::Greyscale(100));

        let style_normal = style::Style::new().with_font_size(12);
        let style_normal_bold = style_normal.bold();
        let mut style_normal_bold_grey = style_normal_bold;
        style_normal_bold_grey.set_color(style::Color::Greyscale(100));

        let mut table = elements::TableLayout::new(vec![1, 3]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(false, false, false));

        let layout = description_iterator
            .fold(elements::LinearLayout::vertical(), |layout, line| {
                layout.element(elements::Paragraph::new(replace_special_chars(line)))
            });

        let to_word = match language {
            "de" => "bis",
            _ => "to",
        };
        let for_word = match language {
            "de" => "für",
            _ => "for",
        };

        table
            .row()
            .element(
                elements::Paragraph::new(format!("{} {} {}", project.from, to_word, project.to))
                    .styled(style_normal_bold_grey)
                    .padded(1),
            )
            .element(
                elements::Paragraph::new(summary)
                    .styled(style_normal_bold)
                    .padded(1),
            )
            .push()
            .expect("Invalid table row");
        table
            .row()
            .element(
                elements::Paragraph::new(format!("{} {} {}", for_word, client_name, roles_string))
                    .styled(style_small_grey)
                    .padded(1),
            )
            .element(layout.padded(1))
            .push()
            .expect("Invalid table row");
        table
            .row()
            .element(Paragraph::new(""))
            .element(
                Paragraph::new(tech_string)
                    .styled(style_small_grey)
                    .padded(1),
            )
            .push()
            .expect("Invalid table row");
        table
            .row()
            .element(Break::new(1))
            .element(Break::new(1))
            .push()
            .expect("");

        if force_page_break {
            doc.push(PageBreak::new());
        }

        doc.push(table);
    }

    doc
}

fn add_header(
    post_address_lines: Vec<String>,
    mailphone_lines: Vec<String>,
    //font_family: FontFamily<Font>,
    logo_scale: Option<f64>,
    font_size: u8,
) -> genpdf::SimplePageDecorator {
    let mut decorator = genpdf::SimplePageDecorator::new();
    decorator.set_margins(10);

    let post_address_paragraphs: Vec<elements::Paragraph> = post_address_lines
        .iter()
        .map(elements::Paragraph::new)
        .collect();

    let mailphone_paragraphs: Vec<elements::Paragraph> = mailphone_lines
        .iter()
        .map(elements::Paragraph::new)
        .collect();

    decorator.set_header(move |_page| {
        let image_result = Image::from_path(IMAGE_PATH_JPG);

        let mut logo_layout = LinearLayout::vertical();

        match image_result {
            Ok(mut image) => {
                if let Some(scale) = logo_scale {
                    image.set_scale(Scale::new(scale, scale))
                };

                logo_layout.push(image);
            }
            _e => {
                // if image is not configured, we just don't show any
                logo_layout.push(Break::new(1));
            }
        }

        let mut post_address_layout = LinearLayout::vertical();
        post_address_layout.push(Break::new(1));
        post_address_paragraphs.iter().for_each(|p| {
            post_address_layout.push(p.to_owned());
        });

        let mut mailphone_layout = LinearLayout::vertical();
        mailphone_layout.push(Break::new(1));
        mailphone_paragraphs.iter().for_each(|p| {
            mailphone_layout.push(p.to_owned());
        });

        let mut table = elements::TableLayout::new(vec![1, 1, 1, 1]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(false, false, false));

        table
            .row()
            .element(logo_layout)
            .element(Break::new(1))
            .element(post_address_layout)
            .element(mailphone_layout)
            .push()
            .expect("Invalid row");

        table
            .row()
            .element(Break::new(1))
            .element(Break::new(1))
            .element(Break::new(1))
            .element(Break::new(1))
            .push()
            .expect("Invalid row");

        table.styled(
            style::Style::new()
                //.with_font_family(font_family)
                .with_font_size(font_size),
        )
    });

    decorator
}

fn get_title(language: &str) -> &str {
    match language {
        "de" => "Projektliste",
        "en" => "Project List",
        y => panic!("invalid language {}", y),
    }
}

fn replace_special_chars(line: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"[\s]+").expect("invalid regex given"); // as the regex pattern doesn't change, it is safe here
    }
    // replaces white-space chars (tabs) with simple blanks
    REGEX.replace_all(line, " ").to_string()
}

fn get_lines_per_page(config: &Config) -> i64 {
    let lines_per_page = match config.get_int("pdf_max_lines_per_page") {
        Ok(number) => number,
        _e => {
            log::warn!(
                "PDF_MAX_LINE_PER_PAGE per page not found in env file. Using default value 30"
            );
            30
        }
    };
    lines_per_page
}

fn get_font(config: &Config) -> String {
    let font_name = match config.get_string("pdf_font") {
        Ok(font_name) => font_name,
        _e => {
            log::warn!("PDF_FONT_NAME not found in env file. Using default font Roboto");
            "Roboto".to_string()
        }
    };
    font_name
}

fn get_env_vars_by_prefix(
    config: &Config,
    numbered_key_prefix: &str,
    language: &str,
    lines: &mut Vec<String>,
) {
    for number in 0..4 {
        let key = format!("{}_{}{}", language, numbered_key_prefix, number);
        let val = config.get_string(&key);

        if val.is_err() {
            log::trace!("key not found. stopping {}", key);
            break;
        }

        let actual_val = val.expect("could not get value"); // safe - checked before
        lines.push(actual_val.as_str().to_owned());
    }
}

fn delete_file_with_delay(file_to_delete: String, delay_in_seconds: u64) {
    tokio::spawn(async move {
        log::debug!("sleep for some time");
        tokio::time::sleep(Duration::from_secs(delay_in_seconds)).await;
        
        log::debug!("now trying to delete the file");

        let path = Path::new(file_to_delete.as_str());
        if path.exists() && path.is_file() {
            match fs::remove_file(&file_to_delete) {
                Ok(_) => {
                    log::info!("file {} deleted.", file_to_delete);
                },
                e => {
                    log::error!(
                        "Could not delete file {}. Error was {:?}",
                        file_to_delete,
                        e
                    );
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_special_chars() {
        assert_eq!(
            "test with a tab - tab was here",
            replace_special_chars("test with a tab  - tab was here"),
            "tabs should have been replaced with simple blanks"
        );
    }
}
