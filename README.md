# MyLauncher

Un launcher de aplicaciones rápido y moderno escrito en Rust con GTK4, diseñado para integrarse perfectamente con XFCE/Wayfire.

## 🚀 Características

- **🎨 Modern UI** con diseño atractivo y transiciones suaves
- **🖼️ Soporte de iconos** usando el tema del sistema (hicolor)
- **⚡ Búsqueda instantánea** mientras escribes
- **🎯 Integración XFCE** - detecta temas automáticamente
- **🔧 Sin configuración compleja** - funciona al instante
- **⌨️ Navegación completa** con teclado y ratón
- **📦 Compatible con Flatpak** y aplicaciones locales

## 🛠️ Construido con

- **Rust** - Rápido y seguro
- **GTK4** - Interfaz moderna y nativa
- **libadwaita** - Componentes de UI consistentes
- **Freedesktop** - Estándar de aplicaciones

## 📂 Funcionalidades

- Búsqueda de aplicaciones por nombre y descripción
- Ejecución con doble click o Enter
- Navegación con flechas
- Escape para cerrar
- Detección automática de aplicaciones instaladas
- Soporte para Flatpak, locales y del sistema

## 🎮 Atajos

- `Super + Espacio` - Abrir launcher
- `Escape` - Cerrar launcher
- `Enter` - Ejecutar aplicación seleccionada
- `Flechas` - Navegar resultados
- `Tipear` - Filtrar aplicaciones

## 🔧 Instalación

```bash
# Clonar repositorio
git clone https://github.com/tu-usuario/mylauncher.git
cd mylauncher

# Construir aplicación
cargo build --release

# Instalar en sistema
sudo cp target/release/mylauncher /usr/local/bin/

# Configurar en Wayfire (opcional)
# Añadir a ~/.config/wayfire.ini:
# command_launcher = /usr/local/bin/mylauncher
```

## 📄 Ejecución

```bash
# Ejecutar directamente
mylauncher

# O con atajo configurado
# Presiona Super + Espacio
```

## 📝 Configuración automática

El launcher detecta automáticamente:
- Tema de iconos del sistema
- Aplicaciones instaladas
- Preferencias del usuario

No requiere configuración manual para funcionar.

---

**Desarrollado con ❤️ en Rust para una experiencia de escritorio fluida y moderna.**%
