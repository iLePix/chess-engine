Still missing

- [x] en passant is still missing and opponent moving cycles
- [ ] 50 move rule, 3-fold repetition need to be implemented // safe all moves
- [x] draws in general
- [ ] premoving
- [x] promoting, kinda just for queen
- [x] check & check-mate
- [x] castling in both direction
- [x] King can currently castle through, even if castling-way is checked!!
- [ ] drag & drop

- [ ] selection red rect smaller


How to run:
* brew install sdl2
Add this line to: ~/.zshenv or ~/.bash_profile, depending on wether u use zsh or bash
* export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"

Install Rust &
* rustup default nightly

At home:
* cargo run


Multiplayer:
* cargo run 81.169.212.158:1337

Press 'C' for switching color theme 
