# Rust-Lab-Spreadsheet-program
COP290 2024-2025 Sem II, C Lab: Spreadsheet program.

--------------------------------
currently:
The project includes a Vim-like text editor with 3 versions:  

1. **Simple version:**  
    - Tested this, features work.
    - Doesnt support :h command.
   ```bash
   cargo run --bin main_simple
   ```  

1. **Simple version: with help**  
    - Added :h to simple version
    - Cudnt test other features whether they broke or not.
   ```bash
   cargo run --bin main_simple
   ```  

2. **Advanced version:**  
   ```bash
   cargo run --bin main_new
   ```  

💡 *Note:*  
- To see list of features, type : 
```bash
:h
```
- The `main_new` version was created to add new features without affecting the original code. Some older features broke during development, so a separate file was made.  
- Both versions share some common features, while others are unique to each.  

### ⚙️ **Setup:**  
To build the project, run:  
```bash
cargo build
```

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
