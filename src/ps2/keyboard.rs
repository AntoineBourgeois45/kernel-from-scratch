use core::sync::atomic::{AtomicBool, Ordering};
use crate::{kprint, vga::terminal::LogLevel};

// États des touches de modification
static SHIFT_PRESSED: AtomicBool = AtomicBool::new(false);
static CTRL_PRESSED: AtomicBool = AtomicBool::new(false);
static ALT_PRESSED: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK: AtomicBool = AtomicBool::new(false);

// Buffer circulaire pour stocker les entrées clavier
const BUFFER_SIZE: usize = 256;
static mut KEY_BUFFER: [char; BUFFER_SIZE] = ['\0'; BUFFER_SIZE];
static mut BUFFER_HEAD: usize = 0;
static mut BUFFER_TAIL: usize = 0;

// Codes de scan des touches de modification
const LSHIFT_PRESS: u8 = 0x2A;
const LSHIFT_RELEASE: u8 = 0xAA;
const RSHIFT_PRESS: u8 = 0x36;
const RSHIFT_RELEASE: u8 = 0xB6;
const CTRL_PRESS: u8 = 0x1D;
const CTRL_RELEASE: u8 = 0x9D;
const ALT_PRESS: u8 = 0x38;
const ALT_RELEASE: u8 = 0xB8;
const CAPS_LOCK_PRESS: u8 = 0x3A;
const CAPS_LOCK_RELEASE: u8 = 0xBA;

// Table de correspondance scancode vers caractère (QWERTY US)
// Table normale (sans shift)
const SCANCODE_TO_ASCII: [char; 128] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08', '\t',
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's',
    'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v',
    'b', 'n', 'm', ',', '.', '/', '\0', '*', '\0', ' ', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0'
];

// Table avec shift ou caps lock
const SHIFT_SCANCODE_TO_ASCII: [char; 128] = [
    '\0', '\x1B', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\x08', '\t',
    'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n', '\0', 'A', 'S',
    'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~', '\0', '|', 'Z', 'X', 'C', 'V',
    'B', 'N', 'M', '<', '>', '?', '\0', '*', '\0', ' ', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0',
    '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0'
];

/// Traite un scancode et le convertit en caractère
pub fn process_scancode(scancode: u8) -> Option<char> {
    // Vérifie si c'est un scancode de relâchement (bit 7 à 1)
    let is_release = (scancode & 0x80) != 0;
    let scancode = scancode & 0x7F; // Masque le bit de relâchement

    // Gestion des touches de modification
    match scancode {
        // Shift gauche
        sc if sc == LSHIFT_PRESS & 0x7F => {
            SHIFT_PRESSED.store(true, Ordering::SeqCst);
            return None;
        }
        sc if sc == LSHIFT_RELEASE & 0x7F => {
            SHIFT_PRESSED.store(false, Ordering::SeqCst);
            return None;
        }
        // Shift droit
        sc if sc == RSHIFT_PRESS & 0x7F => {
            SHIFT_PRESSED.store(true, Ordering::SeqCst);
            return None;
        }
        sc if sc == RSHIFT_RELEASE & 0x7F => {
            SHIFT_PRESSED.store(false, Ordering::SeqCst);
            return None;
        }
        // Ctrl
        sc if sc == CTRL_PRESS & 0x7F => {
            CTRL_PRESSED.store(true, Ordering::SeqCst);
            return None;
        }
        sc if sc == CTRL_RELEASE & 0x7F => {
            CTRL_PRESSED.store(false, Ordering::SeqCst);
            return None;
        }
        // Alt
        sc if sc == ALT_PRESS & 0x7F => {
            ALT_PRESSED.store(true, Ordering::SeqCst);
            return None;
        }
        sc if sc == ALT_RELEASE & 0x7F => {
            ALT_PRESSED.store(false, Ordering::SeqCst);
            return None;
        }
        // Caps Lock (bascule à l'appui)
        sc if sc == CAPS_LOCK_PRESS & 0x7F && !is_release => {
            let current = CAPS_LOCK.load(Ordering::SeqCst);
            CAPS_LOCK.store(!current, Ordering::SeqCst);
            kprint!(LogLevel::Debug, "Caps Lock: {}\n", !current);
            return None;
        }
        _ => {}
    }

    // Si c'est un relâchement de touche, ne pas générer de caractère
    if is_release {
        return None;
    }

    // Vérifier si le scancode est dans la plage valide
    if scancode >= 128 {
        return None;
    }

    // Déterminer si on doit utiliser la table shift
    let shift_active = SHIFT_PRESSED.load(Ordering::SeqCst);
    let caps_active = CAPS_LOCK.load(Ordering::SeqCst);
    
    // Appliquer caps lock uniquement aux lettres (a-z)
    let use_shift_table = if scancode >= 0x10 && scancode <= 0x19 || 
                             scancode >= 0x1E && scancode <= 0x26 ||
                             scancode >= 0x2C && scancode <= 0x32 {
        shift_active != caps_active  // XOR logique
    } else {
        shift_active
    };

    // Sélectionner le caractère en fonction de la table
    let ascii_char = if use_shift_table {
        SHIFT_SCANCODE_TO_ASCII[scancode as usize]
    } else {
        SCANCODE_TO_ASCII[scancode as usize]
    };

    // Appliquer les modificateurs Ctrl et Alt si nécessaire
    if CTRL_PRESSED.load(Ordering::SeqCst) && ascii_char >= 'a' && ascii_char <= 'z' {
        // Ctrl+lettre génère des codes ASCII de contrôle (1-26)
        let ctrl_char = (ascii_char as u8 - b'a' + 1) as char;
        Some(ctrl_char)
    } else {
        // Ignorer les caractères nuls
        if ascii_char == '\0' {
            None
        } else {
            Some(ascii_char)
        }
    }
}

/// Ajoute un caractère au buffer
pub fn enqueue_key(c: char) {
    unsafe {
        let next_head = (BUFFER_HEAD + 1) % BUFFER_SIZE;
        if next_head != BUFFER_TAIL {
            KEY_BUFFER[BUFFER_HEAD] = c;
            BUFFER_HEAD = next_head;
        }
    }
}

/// Récupère un caractère du buffer (ou None si vide)
pub fn dequeue_key() -> Option<char> {
    unsafe {
        if BUFFER_HEAD == BUFFER_TAIL {
            return None; // Buffer vide
        }
        
        let c = KEY_BUFFER[BUFFER_TAIL];
        BUFFER_TAIL = (BUFFER_TAIL + 1) % BUFFER_SIZE;
        Some(c)
    }
}

/// Retourne true si le buffer contient des caractères
pub fn has_key() -> bool {
    unsafe {
        BUFFER_HEAD != BUFFER_TAIL
    }
}

/// Retourne le nombre de caractères dans le buffer
pub fn key_count() -> usize {
    unsafe {
        if BUFFER_HEAD >= BUFFER_TAIL {
            BUFFER_HEAD - BUFFER_TAIL
        } else {
            BUFFER_SIZE - BUFFER_TAIL + BUFFER_HEAD
        }
    }
}

/// Vide le buffer
pub fn clear_buffer() {
    unsafe {
        BUFFER_HEAD = 0;
        BUFFER_TAIL = 0;
    }
}

/// Interface principale pour la gestion du clavier
pub fn handle_keyboard_input(scancode: u8) {
    // Afficher le scancode pour le débogage
    kprint!(LogLevel::Debug, "Scancode: 0x{:02x}\n", scancode);
    
    // Traiter le scancode
    if let Some(c) = process_scancode(scancode) {
        // Ajouter le caractère au buffer
        enqueue_key(c);
        
        // Afficher le caractère tapé
        kprint!(LogLevel::Info, "Key pressed: '{}' ({})\n", 
            if c < ' ' { '?' } else { c },  // Affiche '?' pour les caractères de contrôle
            c as u32
        );
    }
}

/// Lit un caractère (bloquant)
pub fn read_char() -> char {
    loop {
        if let Some(c) = dequeue_key() {
            return c;
        }
        // Attendre qu'une touche soit disponible
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Lit une ligne de texte jusqu'à rencontrer une nouvelle ligne
pub fn read_line(buffer: &mut [u8], max_len: usize) -> usize {
    let mut count = 0;
    
    while count < max_len - 1 {
        let c = read_char();
        
        // Gestion du backspace
        if c == '\x08' && count > 0 {
            count -= 1;
            buffer[count] = 0;
            // Effacer le caractère de l'écran
            kprint!(LogLevel::Default, "\x08 \x08");
            continue;
        }
        
        // Retour à la ligne termine la saisie
        if c == '\n' {
            buffer[count] = b'\n';
            count += 1;
            kprint!(LogLevel::Default, "\n");
            break;
        }
        
        // Ignorer les caractères de contrôle
        if c < ' ' && c != '\t' {
            continue;
        }
        
        // Ajouter le caractère au buffer et l'afficher
        buffer[count] = c as u8;
        count += 1;
        
        // Écho du caractère
        let mut echo_buf = [0u8; 1];
        echo_buf[0] = c as u8;
        unsafe {
            crate::terminal().write(&echo_buf);
        }
    }
    
    // Ajouter le terminateur nul
    buffer[count] = 0;
    
    count
}