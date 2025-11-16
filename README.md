# Rust-Lab-Spreadsheet-program
COP290 2024-2025 Sem II, C Lab: Spreadsheet program.

Demo video:
https://www.youtube.com/watch?v=U-jtmYDXNxA
--------------------------------
for terminal spreadsheet: cargo run --bin sheet 5 6
for vim spreadsheet: cargo run 5 5 --vim
also for vim :
## 📖 **Help Menu**

#### 🛠️ **main_simple_help**
```
📖 Spreadsheet Help Menu  
────────────────────────  
MOVEMENT:  
  h, ←        → Move left  
  l, →        → Move right  
  k, ↑        → Move up  
  j, ↓        → Move down  

EDITING:  
  i           → Enter insert mode  
  ESC         → Exit insert mode  
  x           → Delete character  
  yy          → Copy current line  
  p           → Paste copied line  

COMMANDS:  
  :select <phrase> → Highlight phrase  
  :h              → Open help menu  
  q               → Quit  

────────────────────────  
Press ESC to return to the spreadsheet.  
```

---

#### ⚡ **main_new**
```
📖 Spreadsheet Help Menu  
────────────────────────  
h, l, j, k    → Move cursor  
x             → Delete character  
:b            → Bold current character  
:i            → Italicize current character  
:u            → Underline current character  
:color red    → Change text color to red  
:color green  → Change text color to green  
:color blue   → Change text color to blue  
:reset        → Remove formatting  
q             → Quit  

────────────────────────  
Press ESC to return to the spreadsheet.  
