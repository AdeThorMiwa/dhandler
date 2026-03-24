use std::io::BufWriter;

use anyhow::anyhow;
use printpdf::{BuiltinFont, IndirectFontRef, Line, Mm, PdfDocument, PdfLayerReference, Point};

use super::types::{
    CertificationsSection, Education, EducationSection, ExperienceSection, HeaderSection,
    ProjectsSection, Resume, Section, SkillsSection, SummarySection, WorkExperience,
    ContactInfo,
};

pub trait ResumeTemplate {
    type Output;
    fn render(&self, resume: &Resume) -> anyhow::Result<Self::Output>;
}

pub struct MarkdownTemplate;

impl ResumeTemplate for MarkdownTemplate {
    type Output = String;

    fn render(&self, resume: &Resume) -> anyhow::Result<String> {
        let mut parts: Vec<String> = Vec::new();

        for section in &resume.sections {
            match section {
                Section::Header(h) => {
                    parts.push(format!("# {}", h.name));
                    if let Some(c) = &h.contact {
                        if !c.is_empty() {
                            parts.push(fmt_contact_md(c));
                        }
                    }
                }
                Section::Summary(s) => {
                    parts.push("## Summary".to_string());
                    parts.push(s.content.clone());
                }
                Section::Experience(e) => {
                    parts.push("## Experience".to_string());
                    for exp in &e.entries {
                        parts.push(fmt_experience_heading(exp));
                        for bullet in &exp.highlights {
                            parts.push(format!("- {}", bullet));
                        }
                    }
                }
                Section::Education(e) => {
                    parts.push("## Education".to_string());
                    for edu in &e.entries {
                        parts.push(fmt_education_line(edu));
                    }
                }
                Section::Skills(s) => {
                    parts.push("## Skills".to_string());
                    parts.push(s.skills.join(" · "));
                }
                Section::Projects(p) => {
                    parts.push("## Projects".to_string());
                    for proj in &p.entries {
                        parts.push(format!("### {}", proj.name));
                        parts.push(proj.description.clone());
                        for bullet in &proj.highlights {
                            parts.push(format!("- {}", bullet));
                        }
                    }
                }
                Section::Certifications(c) => {
                    parts.push("## Certifications".to_string());
                    for item in &c.items {
                        parts.push(format!("- {}", item));
                    }
                }
            }
        }

        Ok(parts.join("\n\n"))
    }
}

pub struct PdfTemplate;

impl ResumeTemplate for PdfTemplate {
    type Output = Vec<u8>;

    fn render(&self, resume: &Resume) -> anyhow::Result<Vec<u8>> {
        let mut w = PdfWriter::new(&resume.candidate_name)?;

        for section in &resume.sections {
            match section {
                Section::Header(h)         => w.header(h),
                Section::Summary(s)        => w.summary(s),
                Section::Experience(e)     => w.experience(e),
                Section::Education(e)      => w.education(e),
                Section::Skills(s)         => w.skills(s),
                Section::Projects(p)       => w.projects(p),
                Section::Certifications(c) => w.certifications(c),
            }
        }

        w.finish()
    }
}

const A4_W: f32 = 210.0;
const A4_H: f32 = 297.0;
const MARGIN_X: f32 = 22.0;
const MARGIN_TOP: f32 = 20.0;
const MARGIN_BOTTOM: f32 = 18.0;
const CONTENT_W: f32 = A4_W - 2.0 * MARGIN_X;

const PT_NAME: f32 = 22.0;
const PT_CONTACT: f32 = 9.5;
const PT_SECTION: f32 = 11.0;
const PT_BODY: f32 = 10.0;
const PT_SMALL: f32 = 9.0;

// 1pt ≈ 0.3528mm — used to convert font size to vertical space
const PT_MM: f32 = 0.3528;

struct PdfWriter {
    doc:     printpdf::PdfDocumentReference,
    regular: IndirectFontRef,
    bold:    IndirectFontRef,
    layer:   PdfLayerReference,
    /// Current Y from the bottom of the page, in mm.
    y:       f32,
}

impl PdfWriter {
    fn new(title: &str) -> anyhow::Result<Self> {
        let (doc, page, layer_idx) =
            PdfDocument::new(title, Mm(A4_W), Mm(A4_H), "Layer 1");

        let layer   = doc.get_page(page).get_layer(layer_idx);
        let regular = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| anyhow!("font error: {e}"))?;
        let bold    = doc.add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| anyhow!("font error: {e}"))?;

        Ok(Self { doc, regular, bold, layer, y: A4_H - MARGIN_TOP })
    }

    fn new_page(&mut self) {
        let (page, layer_idx) = self.doc.add_page(Mm(A4_W), Mm(A4_H), "Layer 1");
        self.layer = self.doc.get_page(page).get_layer(layer_idx);
        self.y = A4_H - MARGIN_TOP;
    }

    fn ensure(&mut self, needed_mm: f32) {
        if self.y - needed_mm < MARGIN_BOTTOM {
            self.new_page();
        }
    }

    fn gap(&mut self, mm: f32) {
        self.y -= mm;
    }

    /// Write a single pre-wrapped line and advance Y.
    fn line(&mut self, text: &str, font: &IndirectFontRef, pt: f32, after: f32) {
        let h = pt * PT_MM;
        self.ensure(h + after);
        self.y -= h;
        self.layer.use_text(text, pt, Mm(MARGIN_X), Mm(self.y), font);
        self.y -= after;
    }

    /// Write text with automatic word-wrap.
    fn wrapped(&mut self, text: &str, font: &IndirectFontRef, pt: f32) {
        // Helvetica average character width ≈ 0.52× font size in pt, converted to mm.
        let char_w = pt * PT_MM * 0.52;
        let cap = ((CONTENT_W / char_w) as usize).max(1);

        let mut cur = String::new();
        for word in text.split_whitespace() {
            let needed = if cur.is_empty() { word.len() } else { cur.len() + 1 + word.len() };
            if !cur.is_empty() && needed > cap {
                let snapshot = cur.clone();
                self.line(&snapshot, font, pt, 1.2);
                cur = word.to_string();
            } else {
                if !cur.is_empty() {
                    cur.push(' ');
                }
                cur.push_str(word);
            }
        }
        if !cur.is_empty() {
            self.line(&cur, font, pt, 1.2);
        }
    }

    fn section_title(&mut self, title: &str) {
        self.gap(5.0);
        let bold = self.bold.clone();
        self.line(&title.to_uppercase(), &bold, PT_SECTION, 0.5);
        self.draw_rule();
        self.gap(2.5);
    }

    fn draw_rule(&mut self) {
        use printpdf::{Color, Greyscale};
        let y = self.y;
        let line = Line {
            points: vec![
                (Point::new(Mm(MARGIN_X), Mm(y)), false),
                (Point::new(Mm(A4_W - MARGIN_X), Mm(y)), false),
            ],
            is_closed: false,
        };
        self.layer
            .set_outline_color(Color::Greyscale(Greyscale::new(0.4, None)));
        self.layer.set_outline_thickness(0.4);
        self.layer.add_line(line);
        self.y -= 1.5;
    }

    fn header(&mut self, h: &HeaderSection) {
        let bold    = self.bold.clone();
        let regular = self.regular.clone();
        self.line(&h.name, &bold, PT_NAME, 2.5);
        if let Some(c) = &h.contact {
            if !c.is_empty() {
                self.line(&c.display(), &regular, PT_CONTACT, 1.0);
            }
        }
    }

    fn summary(&mut self, s: &SummarySection) {
        self.section_title("Summary");
        let regular = self.regular.clone();
        self.wrapped(&s.content, &regular, PT_BODY);
    }

    fn experience(&mut self, e: &ExperienceSection) {
        self.section_title("Experience");
        for exp in &e.entries {
            self.experience_entry(exp);
        }
    }

    fn experience_entry(&mut self, exp: &WorkExperience) {
        let bold    = self.bold.clone();
        let regular = self.regular.clone();
        self.line(&fmt_experience_heading(exp), &bold, PT_BODY, 1.0);
        for bullet in &exp.highlights {
            let text = format!("\u{2022}  {}", bullet); // •
            self.wrapped(&text, &regular, PT_SMALL);
        }
        self.gap(3.0);
    }

    fn education(&mut self, e: &EducationSection) {
        self.section_title("Education");
        let bold = self.bold.clone();
        for edu in &e.entries {
            self.line(&fmt_education_line(edu), &bold, PT_BODY, 2.0);
        }
    }

    fn skills(&mut self, s: &SkillsSection) {
        self.section_title("Skills");
        let regular = self.regular.clone();
        self.wrapped(&s.skills.join("  ·  "), &regular, PT_BODY);
    }

    fn projects(&mut self, p: &ProjectsSection) {
        self.section_title("Projects");
        for proj in &p.entries {
            let bold    = self.bold.clone();
            let regular = self.regular.clone();
            self.line(&proj.name, &bold, PT_BODY, 1.0);
            self.wrapped(&proj.description, &regular, PT_SMALL);
            for bullet in &proj.highlights {
                let text = format!("\u{2022}  {}", bullet);
                self.wrapped(&text, &regular, PT_SMALL);
            }
            self.gap(3.0);
        }
    }

    fn certifications(&mut self, c: &CertificationsSection) {
        self.section_title("Certifications");
        let regular = self.regular.clone();
        for item in &c.items {
            self.line(&format!("\u{2022}  {}", item), &regular, PT_BODY, 1.5);
        }
    }

    fn finish(self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        self.doc
            .save(&mut BufWriter::new(&mut bytes))
            .map_err(|e| anyhow!("PDF save failed: {e}"))?;
        Ok(bytes)
    }
}

/// Renders contact info as a Markdown line with labelled links where applicable.
fn fmt_contact_md(c: &ContactInfo) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = &c.email        { parts.push(format!("[{}](mailto:{})", v, v)); }
    if let Some(v) = &c.phone        { parts.push(v.clone()); }
    if let Some(v) = &c.linkedin_url { parts.push(format!("[LinkedIn]({})", v)); }
    if let Some(v) = &c.github_url   { parts.push(format!("[GitHub]({})", v)); }
    if let Some(v) = &c.website_url  { parts.push(format!("[Website]({})", v)); }
    if let Some(v) = &c.other        { parts.push(v.clone()); }
    parts.join("  ·  ")
}

fn fmt_experience_heading(exp: &WorkExperience) -> String {
    match &exp.dates {
        Some(d) => format!("{} — {} | {}", exp.title, exp.company, d),
        None    => format!("{} — {}", exp.title, exp.company),
    }
}

fn fmt_education_line(edu: &Education) -> String {
    match (&edu.degree, &edu.dates) {
        (Some(d), Some(dt)) => format!("{} — {} | {}", edu.institution, d, dt),
        (Some(d), None)     => format!("{} — {}", edu.institution, d),
        (None, Some(dt))    => format!("{} | {}", edu.institution, dt),
        (None, None)        => edu.institution.clone(),
    }
}
