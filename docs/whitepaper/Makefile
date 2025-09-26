# SolanaVault Whitepaper Build System
# Requires: pdflatex, bibtex, tikz, pgfplots

PAPER = solanavault_whitepaper
BIBTEX = references

.PHONY: all clean open install-deps check-deps

# Default target
all: check-deps $(PAPER).pdf

# Compile the paper
$(PAPER).pdf: $(PAPER).tex $(BIBTEX).bib
	@echo "🔨 Building SolanaVault whitepaper..."
	pdflatex $(PAPER).tex
	@echo "📚 Processing bibliography..."
	bibtex $(PAPER)
	@echo "🔄 First recompilation..."
	pdflatex $(PAPER).tex
	@echo "🔄 Final recompilation..."
	pdflatex $(PAPER).tex
	@echo "✅ Whitepaper compiled successfully: $(PAPER).pdf"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	rm -f *.aux *.bbl *.blg *.log *.out *.toc *.lot *.lof *.fls *.fdb_latexmk *.synctex.gz *.nav *.snm *.vrb
	@echo "✅ Cleanup complete"

# Clean everything including PDF
distclean: clean
	rm -f $(PAPER).pdf

# Open the compiled PDF (Linux)
open: $(PAPER).pdf
	xdg-open $(PAPER).pdf &

# Open the compiled PDF (macOS)
open-mac: $(PAPER).pdf
	open $(PAPER).pdf

# Check for required dependencies
check-deps:
	@echo "🔍 Checking LaTeX dependencies..."
	@command -v pdflatex >/dev/null 2>&1 || { echo "❌ pdflatex not found. Please install texlive-latex-base"; exit 1; }
	@command -v bibtex >/dev/null 2>&1 || { echo "❌ bibtex not found. Please install texlive-bibtex-extra"; exit 1; }
	@echo "✅ Dependencies check passed"

# Install dependencies (Ubuntu/Debian)
install-deps:
	@echo "📦 Installing LaTeX dependencies..."
	sudo apt-get update
	sudo apt-get install -y texlive-latex-base texlive-latex-extra texlive-bibtex-extra texlive-science texlive-pictures
	@echo "✅ Dependencies installed"

# Install dependencies (macOS with Homebrew)
install-deps-mac:
	@echo "📦 Installing LaTeX dependencies for macOS..."
	brew install --cask mactex
	@echo "✅ MacTeX installed. You may need to restart your terminal."

# Quick build without bibliography (for rapid iteration)
quick: $(PAPER).tex
	@echo "⚡ Quick build (no bibliography)..."
	pdflatex $(PAPER).tex
	@echo "✅ Quick build complete"

# Word count (approximately)
wordcount: $(PAPER).tex
	@echo "📊 Approximate word count:"
	@texcount -1 -sum $(PAPER).tex 2>/dev/null || echo "texcount not available, using wc fallback:"
	@cat $(PAPER).tex | wc -w

# Show paper statistics
stats: $(PAPER).tex
	@echo "📈 Paper Statistics:"
	@echo "  Lines: $$(cat $(PAPER).tex | wc -l)"
	@echo "  Words (approx): $$(cat $(PAPER).tex | wc -w)"
	@echo "  Characters: $$(cat $(PAPER).tex | wc -c)"
	@echo "  Sections: $$(grep -c '\\section{' $(PAPER).tex)"
	@echo "  Figures: $$(grep -c '\\begin{figure}' $(PAPER).tex)"
	@echo "  Tables: $$(grep -c '\\begin{table}' $(PAPER).tex)"
	@echo "  References: $$(grep -c '@' $(BIBTEX).bib)"

# Help target
help:
	@echo "📖 SolanaVault Whitepaper Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  all           - Build the complete whitepaper (default)"
	@echo "  quick         - Quick build without bibliography"
	@echo "  clean         - Remove build artifacts"
	@echo "  distclean     - Remove all generated files including PDF"
	@echo "  open          - Open the PDF (Linux)"
	@echo "  open-mac      - Open the PDF (macOS)"
	@echo "  check-deps    - Check for required dependencies"
	@echo "  install-deps  - Install dependencies (Ubuntu/Debian)"
	@echo "  install-deps-mac - Install dependencies (macOS)"
	@echo "  wordcount     - Show approximate word count"
	@echo "  stats         - Show paper statistics"
	@echo "  help          - Show this help message"
	@echo ""
	@echo "Example usage:"
	@echo "  make all      # Build the whitepaper"
	@echo "  make quick    # Quick iteration during writing"
	@echo "  make clean    # Clean up before committing"
	@echo "  make open     # View the compiled paper"