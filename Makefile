default: help

help:
	@echo "to update_readme, use the command: 'make update_readme'"
	@echo "to build the projects, use the Makefile in the respective directory"

update_readme:
	./u/update-readme > README.md
