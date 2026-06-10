<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | [简体中文](README-zh_cn.md) | [繁體中文](README-zh_tw.md) | Español (España) | [Español (Latinoamérica)](README-es_419.md) | [Français](README-fr_fr.md) | [Português (Brasil)](README-pt_br.md) | [Português (Portugal)](README-pt_pt.md)

Mod de mejora y traducción del juego para UM:PD. HachimiRedux es un fork de Hachimi con un plugin de seguimiento de entrenamiento integrado en el juego y un SDK de plugins nativo rediseñado.

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

## Tabla de contenidos

- [Por favor, no enlaces a este repositorio ni a la web de Hachimi](#️-por-favor-no-enlaces-a-este-repositorio-ni-a-la-web-de-hachimi)
- [Incompatible con los plugins de Hachimi original](#️-incompatible-con-los-plugins-de-hachimi-original)
- [Características](#características)
- [Instalación](#instalación)
  - [Instalar con el instalador (recomendado)](#instalar-con-el-instalador-recomendado)
  - [Compilar desde el código fuente (avanzado)](#compilar-desde-el-código-fuente-avanzado)
- [Resolución de problemas](#resolución-de-problemas)
- [Agradecimientos especiales](#agradecimientos-especiales)
- [Licencia](#licencia)

# ⚠️ Por favor, no enlaces a este repositorio ni a la web de Hachimi
Entendemos que quieras ayudar a la gente a instalar Hachimi y a tener una mejor experiencia jugando. Sin embargo, este proyecto va, por su propia naturaleza, en contra de las condiciones de servicio del juego, y los desarrolladores del juego con toda seguridad querrían eliminarlo si llegaran a tener conocimiento de él.

Compartirlo en tus servicios de chat privados y mediante mensajes directos está bien, pero te pedimos humildemente que evites compartir enlaces a este proyecto en sitios de cara al público, o a cualquiera de las herramientas implicadas.

O compártelos y arruínaselo a la docena de usuarios de Hachimi. Tú decides.

### Si vas a compartirlo de todas formas
Haz lo que debas, pero te pedimos respetuosamente que intentes etiquetar el juego como «UM:PD» o «The Honse Game» en lugar del nombre real del juego, para evitar el rastreo de los motores de búsqueda.

# ⚠️ Incompatible con los plugins de Hachimi original
Este fork incluye su propia API nativa de plugins (host API v9). **Los plugins creados para el Hachimi original no son compatibles con HachimiRedux**, y el plugin de seguimiento de entrenamiento que se distribuye aquí no se cargará en el Hachimi original. Usa preferiblemente DLL compiladas a partir de este repositorio, y úsalas juntas. Mezclar compilaciones puede provocar fallos al cargar o cierres inesperados del juego.

## Compatibilidad con plugins heredados (opcional, limitada)
Los plugins sin manifiesto y con ABI heredada (p. ej. los volcadores de datos del Hachimi original) pueden cargarse a través de una **vía de compatibilidad opcional**. Añade la DLL a una lista blanca `legacy_libraries` en `config.json`, además de a `load_libraries`:

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

Un plugin heredado solo necesita exportar `hachimi_init`; el host omite su comprobación habitual de manifiesto/ABI y lo carga por confianza. Este soporte es **limitado y no oficial**:

- El plugin debe depender **únicamente del prefijo estable de la vtable** de la API del host. Cualquier cosa más allá de eso es comportamiento indefinido y puede provocar el cierre del juego.
- El host **no puede validar, rastrear ni descargar** un plugin heredado ni sus hooks de IL2CPP. La DLL permanece mapeada durante toda la vida del proceso.
- Se registra una advertencia cada vez que un plugin se carga por esta vía.
- Las entradas de `legacy_libraries` también deben aparecer en `load_libraries`.

En caso de duda, recompila el plugin contra este repositorio (host API v9) en lugar de depender de la vía heredada.

# Características
- **Traducciones de alta calidad:** Hachimi incluye funciones de traducción avanzadas que ayudan a que las traducciones se sientan más naturales (formas plurales, números ordinales, etc.) y evitan introducir artefactos en la interfaz. Además, permite traducir la mayoría de los componentes del juego; ¡sin necesidad de parchear assets manualmente!

    Componentes soportados:
    - Texto de la interfaz
    - master.mdb (nombres de habilidades, descripciones de habilidades, etc.)
    - Historias de carreras
    - Historia principal / diálogos del Home
    - Letras de canciones
    - Reemplazo de texturas
    - Reemplazo de atlas de sprites

    Además, Hachimi no ofrece funciones de traducción solo para un único idioma; ha sido diseñado para ser totalmente configurable para cualquier idioma.

- **Configuración sencilla:** Solo enchufar y listo. Toda la configuración se realiza dentro del propio juego, sin necesidad de aplicaciones externas.
- **Actualización automática de traducciones:** El actualizador de traducciones integrado te permite jugar con normalidad mientras se actualiza, y la recarga dentro del juego cuando termina, ¡sin necesidad de reiniciar!
- **Interfaz gráfica integrada:** Incluye un editor de configuración para que modifiques los ajustes sin tener que salir siquiera del juego.
- **Ajustes gráficos:** Puedes ajustar la configuración gráfica del juego para aprovechar al máximo las características de tu dispositivo, como el desbloqueo de FPS y el escalado de resolución.
- **Solo Windows:** Creado específicamente para la versión de Windows (Steam) del juego. **HachimiRedux no soporta Android por decisión propia**: se centra únicamente en el cliente de Windows, y no hay planes de añadir ni mantener una compilación para Android.

# Instalación

La forma más fácil de instalar HachimiRedux es con el **instalador** de la [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases): configura por ti el mod principal y el plugin opcional Training Tracker, sin copiar archivos a mano ni editar JSON. Si prefieres compilarlo tú mismo, consulta [Compilar desde el código fuente](#compilar-desde-el-código-fuente-avanzado).

HachimiRedux es el mod principal (se carga como `cri_mana_vpx.dll`); el **Training Tracker** es un plugin DLL opcional que carga el mod principal. Ambos provienen de la misma compilación.

El directorio del juego es la carpeta de instalación de Steam, p. ej.
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`.

## Instalar con el instalador (recomendado)

1. Descarga el `hachimi_installer.exe` más reciente de la [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases).
2. Ejecútalo. El instalador detecta automáticamente tu directorio del juego de Steam; si no puede, selecciónalo manualmente (la ruta por defecto está más arriba).
3. Elige tu idioma. Para obtener el Training Tracker dentro del juego, deja marcada la casilla **«Install Training Tracker plugin»** (activada por defecto).
4. Haz clic en **Install**. El instalador hace una copia de seguridad del `cri_mana_vpx.dll` original, instala el mod y crea `config.json` por ti.
5. Inicia el juego. Pulsa la tecla de menú —por defecto la **flecha derecha**— para abrir la interfaz dentro del juego.

Para actualizar o eliminar HachimiRedux más adelante, vuelve a ejecutar el instalador (ofrece una opción de desinstalación).

## Compilar desde el código fuente (avanzado)

Este repositorio es un workspace de Cargo. Desde la raíz del repositorio:

```sh
# Mod principal
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Plugin Training Tracker
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## Instalar HachimiRedux (núcleo)

El juego carga el mod a través de la DLL del renderizador `cri_mana_vpx.dll`.

1. En el directorio del juego, haz una copia de seguridad del `cri_mana_vpx.dll` original como `cri_mana_vpx.dll.backup` (hazlo una sola vez: nunca sobrescribas la copia de seguridad después).
2. Copia `target/release/hachimi.dll` en el directorio del juego y renómbralo a `cri_mana_vpx.dll`.
3. Inicia el juego. Pulsa la tecla de menú —por defecto es la **flecha derecha**— para abrir la interfaz dentro del juego. La pantalla de inicio muestra la tecla actual, y puedes reasignarla desde la interfaz gráfica dentro del juego.

Los ajustes del mod se guardan en `config.json`, dentro del directorio de datos del juego, que es la **subcarpeta `hachimi` del directorio del juego** (p. ej. `…\UmamusumePrettyDerby\hachimi\config.json`). El instalador la crea automáticamente / en el primer arranque; todo lo demás se configura desde la interfaz gráfica dentro del juego.

## Instalar el plugin Training Tracker

Los plugins son DLL nativas que el mod principal carga al arrancar desde la raíz del directorio del juego.

1. Instala primero el núcleo de HachimiRedux (más arriba).
2. Copia `target/release/hachimi_training_tracker.dll` en la raíz del directorio del juego (la misma carpeta que `cri_mana_vpx.dll`). Nota: la DLL del plugin va en la **raíz** del juego, mientras que `config.json` está en la subcarpeta `hachimi`.
3. Añade la DLL a la lista `load_libraries` en `config.json` (`<game_dir>\hachimi\config.json`):

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. Inicia el juego. El tracker aparece como una página en la pestaña Plugins y como un panel de superposición flotante. Consulta [docs/plugin-sdk.md](docs/plugin-sdk.md) para saber cómo funcionan los plugins.

## Despliegue automatizado (Windows, desde el código fuente)

Desde la raíz del repositorio, el script auxiliar compila y copia ambas DLL en el directorio del juego:

```powershell
.\scripts\deploy-windows.ps1 -Build
```

Sustituye la carpeta del juego si no está en la ruta de Steam por defecto:

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

El script copia `hachimi.dll` → `cri_mana_vpx.dll` y la DLL del training tracker en el directorio del juego, y nunca modifica `cri_mana_vpx.dll.backup`.

# Resolución de problemas

## El juego se cierra al iniciar / se comporta de forma extraña

Con diferencia, la causa más habitual es **acumular varios mods o inyectores de DLL** en la carpeta del juego. Cada uno engancha el renderizado/runtime del juego y compiten entre sí. HachimiRedux avisa de esto dentro del juego (una notificación + el `hachimi.log`) y el instalador avisa antes de instalar, pero debes eliminar los demás tú mismo:

- Mantén **solo** HachimiRedux: `cri_mana_vpx.dll` y cualquier plugin compilado con HachimiRedux (p. ej. `hachimi_training_tracker.dll`).
- Elimina de la carpeta del juego otras superposiciones/inyectores, como DLL de proxy-loader que no deberían estar ahí (`version.dll`, `winhttp.dll`, `dxgi.dll`, `d3d11.dll`, `dinput8.dll`, …) y superposiciones con nombre (`horseACT.dll`, `heaven_overlay.dll`, …).
- **Solo los plugins compilados con HachimiRedux** pertenecen a `load_libraries`. No añadas ahí superposiciones de terceros: no son plugins de HachimiRedux y serán rechazadas (con un aviso dentro del juego) o pueden provocar el cierre del juego.

## Dónde está cada cosa

- `cri_mana_vpx.dll` y las DLL de plugins: el directorio **raíz** del juego.
- `config.json` y otros datos del mod: la **subcarpeta `hachimi`** del directorio del juego (`<game_dir>\hachimi\config.json`).
- Log del mod: `hachimi.log` en la raíz del juego (activa `enable_file_logging` en `config.json`).
- Log del juego: `%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`.

## Recopilar diagnósticos

- Dentro del juego: abre el menú (flecha derecha por defecto) → **Config** → **Save diagnostics report**. Esto escribe `hachimi_diagnostics.txt` en la carpeta del juego.
- Instalador: ejecuta `installer collect-logs` para reunir `config.json`, `hachimi.log` y un informe de conflictos en `%TEMP%\hachimi_diagnostics`.

# Agradecimientos especiales

HachimiRedux es un fork construido sobre el trabajo de:

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) — el proyecto original en el que se basa. Si te interesa el proyecto original, únete a [su servidor de Discord](https://discord.gg/YjBgmuqqYr).
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) — el fork centrado en Windows/Steam del que parte HachimiRedux.

A su vez, estos proyectos han sido la base del desarrollo de Hachimi; sin ellos, Hachimi nunca habría existido en su forma actual:

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# Licencia
[GNU GPLv3](LICENSE)
