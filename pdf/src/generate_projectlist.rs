use config::Config;
use entities::project::ProjectTuple;
use genpdf::elements::{Break, Image, LinearLayout, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::{elements, style};
use genpdf::{Alignment, Element, Scale};
use lazy_static::lazy_static;
use regex::Regex;

const IMAGE_PATH_JPG: &str = "server/static/images/logo.jpg";

pub fn generate_pdf(
    config: &Config,
    project_list: Vec<ProjectTuple>,
    target_file_name: String,
    language: &str,
) -> Result<(), genpdf::error::Error> {
    let font_dir = "./pdf/fonts";
    let font_roboto_str = "Roboto";

    let title = match language {
        "de" => "Projektliste",
        _ => "Project List",
    };

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

    let font_family_roboto = genpdf::fonts::from_files(font_dir, font_roboto_str, None)
        .expect("Failed to load font family");

    let document = generate_document(
        font_family_roboto,
        title.to_string(),
        project_list,
        language,
        post_address_lines,
        mailphone_lines,
        logo_scale,
    )?;

    document.render_to_file(target_file_name)
}

fn generate_document(
    font_family: FontFamily<FontData>,
    title: String,
    project_list: Vec<ProjectTuple>,
    language: &str,
    post_address_lines: Vec<String>,
    mailphone_lines: Vec<String>,
    logo_scale: Option<f64>,
) -> Result<genpdf::Document, genpdf::error::Error> {
    
    let projects_rendered = render_projects(project_list, language);

    let mut doc = genpdf::Document::new(font_family);
    doc.set_title(&title);
    doc.push(elements::Break::new(1.5));
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

    // now add the project tables
    for project_rendered in projects_rendered {
        doc.push(project_rendered);
    }

    Ok(doc)
}

fn render_projects(project_tuples: Vec<ProjectTuple>, language: &str) -> Vec<TableLayout> {
    let mut list: Vec<TableLayout> = Vec::new();

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

        let iter = description.split('\n');

        let style_small = style::Style::new().with_font_size(10);
        let mut style_small_grey = style_small;
        style_small_grey.set_color(style::Color::Greyscale(100));

        let style_normal = style::Style::new().with_font_size(12);
        let style_normal_bold = style_normal.bold();
        let mut style_normal_bold_grey = style_normal_bold;
        style_normal_bold_grey.set_color(style::Color::Greyscale(100));

        let mut table = elements::TableLayout::new(vec![1, 3]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(false, false, false));

        let layout = iter.fold(elements::LinearLayout::vertical(), |layout, line| {
            layout.element(elements::Paragraph::new(replace_special_chars(line)))
        });

        table
            .row()
            .element(
                elements::Paragraph::new(format!("{} bis {}", project.from, project.to))
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
                elements::Paragraph::new(format!("für {} {}", client_name, roles_string))
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

        list.push(table);
    }

    list
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
                log::info!("scale is {:?}", logo_scale);

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

fn replace_special_chars(line: &str) -> String {
    lazy_static! {
        static ref REGEX: Regex = Regex::new(r"[\s]+").expect("invalid regex given"); // as the regex pattern doesn't change, it is safe here
    }
    // replaces white-space chars (tabs) with simple blanks
    REGEX.replace_all(line, " ").to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_special_chars() {
        assert_eq!(
            "test with a tab - tab was here",
            replace_special_chars("test with a tab     - tab was here"),
            "tabs should have been replaced with simple blanks"
        );
    }
}
