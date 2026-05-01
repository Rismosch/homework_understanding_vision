#!/usr/bin/env bash
set -e

# Project name = current folder
PROJECT_DIR="$(pwd)"
VENV_DIR=".venv"

echo "Creating virtual environment in $PROJECT_DIR/$VENV_DIR"

# Create venv
python -m venv "$VENV_DIR"

# Activate it
source "$VENV_DIR/Scripts/activate"

# Upgrade pip (important, avoids weird issues)
python -m pip install --upgrade pip

# Install common packages (adjust if needed)
pip install matplotlib numpy scipy

echo ""
echo "Done."
echo "To activate later, run:"
echo "  source $VENV_DIR/bin/activate"