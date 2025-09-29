# SolanaVault Whitepaper Makefile
# Builds the whitepaper PDF from LaTeX sources

# Main LaTeX file
MAIN = solanavault_whitepaper

# LaTeX compiler
LATEX = pdflatex
BIBTEX = bibtex

# Build flags
LATEX_FLAGS = -interaction=nonstopmode -halt-on-error

# Output directory
BUILD_DIR = build

.PHONY: all clean pdf view help

# Default target
all: pdf

# Build PDF
pdf: $(MAIN).pdf

$(MAIN).pdf: $(MAIN).tex references.bib
	@echo "Building SolanaVault whitepaper..."
	@mkdir -p $(BUILD_DIR)

	# First pass
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex

	# Process bibliography
	$(BIBTEX) $(MAIN)

	# Second pass (resolve citations)
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex

	# Third pass (resolve references)
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex

	@echo "✅ PDF built successfully: $(MAIN).pdf"

# Quick build (single pass, for development)
quick: $(MAIN).tex
	@echo "Quick build (single pass)..."
	$(LATEX) $(LATEX_FLAGS) $(MAIN).tex
	@echo "✅ Quick build complete"

# View the PDF (if available)
view: $(MAIN).pdf
	@if command -v evince >/dev/null 2>&1; then \
		evince $(MAIN).pdf & \
	elif command -v open >/dev/null 2>&1; then \
		open $(MAIN).pdf; \
	elif command -v xdg-open >/dev/null 2>&1; then \
		xdg-open $(MAIN).pdf; \
	else \
		echo "No PDF viewer found. Please open $(MAIN).pdf manually."; \
	fi

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.lof *.lot *.fls *.fdb_latexmk *.synctex.gz
	rm -rf $(BUILD_DIR)
	@echo "✅ Clean complete"

# Clean everything including PDF
distclean: clean
	rm -f $(MAIN).pdf
	@echo "✅ All files cleaned"

# Check LaTeX installation
check:
	@echo "Checking LaTeX installation..."
	@which $(LATEX) > /dev/null || (echo "❌ pdflatex not found. Please install LaTeX." && exit 1)
	@which $(BIBTEX) > /dev/null || (echo "❌ bibtex not found. Please install LaTeX." && exit 1)
	@echo "✅ LaTeX installation OK"

# Show help
help:
	@echo "SolanaVault Whitepaper Build System"
	@echo "=================================="
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build the complete PDF (default)"
	@echo "  pdf      - Build the complete PDF with bibliography"
	@echo "  quick    - Quick build (single pass, no bibliography)"
	@echo "  view     - Open the PDF in default viewer"
	@echo "  clean    - Remove build artifacts"
	@echo "  distclean- Remove all generated files including PDF"
	@echo "  check    - Check LaTeX installation"
	@echo "  help     - Show this help message"
	@echo ""
	@echo "Requirements:"
	@echo "  - pdflatex (from TeX Live or MiKTeX)"
	@echo "  - bibtex"
	@echo ""
	@echo "Usage:"
	@echo "  make          # Build PDF"
	@echo "  make quick    # Quick development build"
	@echo "  make view     # Build and view PDF"
	@echo "  make clean    # Clean up"

# Development target - rebuild on file changes (requires inotify-tools)
watch:
	@echo "Watching for changes... (Ctrl+C to stop)"
	@echo "Requires: inotify-tools (sudo apt install inotify-tools)"
	@while inotifywait -e modify $(MAIN).tex references.bib 2>/dev/null; do \
		echo "Files changed, rebuilding..."; \
		make quick; \
	done

# Archive the whitepaper sources
archive:
	@echo "Creating source archive..."
	@DATE=$$(date +%Y%m%d); \
	tar -czf solanavault_whitepaper_sources_$$DATE.tar.gz \
		$(MAIN).tex references.bib Makefile README.md IMPLEMENTATION_AUDIT.md \
		--exclude='*.pdf' --exclude='*.aux' --exclude='*.log' \
		--exclude='*.bbl' --exclude='*.blg' --exclude='*.out'
	@echo "✅ Source archive created"

# Word count (approximate)
wordcount: $(MAIN).tex
	@echo "Approximate word count:"
	@detex $(MAIN).tex | wc -w

# Statistics
stats: $(MAIN).pdf
	@echo "Document Statistics:"
	@echo "==================="
	@echo "Pages: $$(pdfinfo $(MAIN).pdf 2>/dev/null | grep Pages | awk '{print $$2}' || echo 'Unknown')"
	@echo "Size: $$(du -h $(MAIN).pdf | cut -f1)"
	@echo "Word count (approx): $$(detex $(MAIN).tex 2>/dev/null | wc -w || echo 'Unknown')"
	@echo "References: $$(grep -c '@' references.bib || echo 'Unknown')"