#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Domain Test Package Initializer
Ensures dynamic sys.path resolution for pos_core, sanitizer, and validators imports.
"""

import sys
from pathlib import Path

# Add scripts directory to sys.path dynamically
scripts_dir = str(Path(__file__).resolve().parent.parent)
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)
