#!/bin/bash

# Set up the Python venv for lxml reference tests using uv.
# The Rust lxml tests expect the venv at tests/lxml_tests/.venv/bin/python.
cd /workspaces/Skyscraper/tests/lxml_tests && uv sync
