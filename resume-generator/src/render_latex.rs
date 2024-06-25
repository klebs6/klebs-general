crate::ix!();

pub const LATEX_POSTAMBLE: &str = r"\end{document}";

pub trait RenderLatex {
    fn latex(&self) -> String;

    fn begin_document(&self) -> String {
        indoc! {r#"
            \documentclass[a4paper,10pt]{article}
            \usepackage[utf8]{inputenc}
            \usepackage{geometry}
            \usepackage{enumitem}
            \usepackage{hyperref}
            \usepackage{titlesec}
            \usepackage{needspace}
            \usepackage{parskip}

            % Adjusting the margins
            \geometry{left=1in, right=1in, top=1in, bottom=1in}

            % Customizing sections
            \titleformat{\section}{\large\bfseries}{}{0em}{}[\titlerule]
            \titleformat{\subsection}{\bfseries}{}{0em}{}

            \setlist[itemize]{topsep=0pt, partopsep=0pt, parsep=0pt, itemsep=4pt}

            \begin{document}
        "#}.to_string()
    }

    fn end_document(&self) -> String {
        LATEX_POSTAMBLE.to_string()
    }
}
