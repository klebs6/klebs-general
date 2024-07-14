OUTPUT_DIRECTORY := renders

TEMPLATE_TEX := template.tex
CHRIS_TEX    := chris.tex

PDFLATEX := pdflatex --output-directory=renders

default: zorc

template:
	cargo run --example example_template_resume -- --output-directory $(OUTPUT_DIRECTORY) --output-filename $(TEMPLATE_TEX)
	open $(OUTPUT_DIRECTORY)/$(TEMPLATE_TEX)

zorc:
	cargo run --example example_zorc_resume -- --output-directory $(OUTPUT_DIRECTORY) --output-filename $(CHRIS_TEX)
	open $(OUTPUT_DIRECTORY)/$(CHRIS_TEX)

chris:
	cargo run --example example_chris_resume -- --output-directory $(OUTPUT_DIRECTORY) --output-filename $(CHRIS_TEX)
	open $(OUTPUT_DIRECTORY)/$(CHRIS_TEX)

nuke:
	rm $(OUTPUT_DIRECTORY)/*
