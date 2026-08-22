// A janela nao pode piscar um console atras antes de aparecer.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    kontro_lib::executar()
}
