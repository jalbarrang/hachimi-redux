<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | [简体中文](README-zh_cn.md) | [繁體中文](README-zh_tw.md) | [Español (España)](README-es_es.md) | [Español (Latinoamérica)](README-es_419.md) | [Français](README-fr_fr.md) | [Português (Brasil)](README-pt_br.md) | Português (Portugal)

Mod de melhoria e tradução do jogo para UM:PD. O HachimiRedux é um fork do Hachimi com um plugin de acompanhamento de treino integrado no jogo e um SDK de plugins nativo reformulado.

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

## Índice

- [Por favor, não crie ligações para este repositório nem para o site do Hachimi](#️-por-favor-não-crie-ligações-para-este-repositório-nem-para-o-site-do-hachimi)
- [Incompatível com os plugins do Hachimi original](#️-incompatível-com-os-plugins-do-hachimi-original)
- [Funcionalidades](#funcionalidades)
- [Instalação](#instalação)
  - [Instalar com o instalador (recomendado)](#instalar-com-o-instalador-recomendado)
  - [Compilar a partir do código-fonte (avançado)](#compilar-a-partir-do-código-fonte-avançado)
- [Resolução de problemas](#resolução-de-problemas)
- [Agradecimentos especiais](#agradecimentos-especiais)
- [Licença](#licença)

# ⚠️ Por favor, não crie ligações para este repositório nem para o site do Hachimi
Compreendemos que queira ajudar as pessoas a instalar o Hachimi e a ter uma melhor experiência a jogar. No entanto, este projeto vai, por natureza, contra os termos de serviço do jogo, e os programadores do jogo certamente iriam querer que ele desaparecesse caso viessem a tomar conhecimento dele.

Partilhar nos seus serviços de chat privados e por mensagens diretas não há problema, mas pedimos humildemente que evite partilhar ligações para este projeto em sites públicos, ou para qualquer uma das ferramentas envolvidas.

Ou partilhe à mesma e estrague tudo para a dúzia de utilizadores do Hachimi. A escolha é sua.

### Se for partilhar de qualquer forma
Faça o que tiver de fazer, mas pedimos respeitosamente que tente identificar o jogo como "UM:PD" ou "The Honse Game" em vez do nome real do jogo, para evitar a indexação pelos motores de busca.

# ⚠️ Incompatível com os plugins do Hachimi original
Este fork inclui a sua própria API nativa de plugins (host API v16). **Os plugins criados para o Hachimi original não são compatíveis com o HachimiRedux**, e o plugin de acompanhamento de treino distribuído aqui não irá carregar no Hachimi original. Prefira DLL compiladas a partir deste repositório, e utilize-as em conjunto. Misturar builds pode falhar ao carregar ou bloquear o jogo.

## Compatibilidade com plugins legados (opcional, limitada)
Os plugins sem manifesto e com ABI legada (por exemplo, os data-dumpers do Hachimi original) podem ser carregados através de um **caminho de compatibilidade opcional**. Adicione a DLL a uma lista de permissões `legacy_libraries` no `config.json`, além de `load_libraries`:

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

Um plugin legado só precisa de exportar `hachimi_init`; o host ignora a sua verificação habitual de manifesto/ABI e carrega-o por confiança. Este suporte é **limitado e sem garantias**:

- O plugin deve depender **apenas do prefixo estável da vtable** da API do host. Tudo o que vá além disso é comportamento indefinido e pode bloquear o jogo.
- O host **não consegue validar, acompanhar nem descarregar** um plugin legado nem os seus hooks de IL2CPP. A DLL permanece mapeada durante todo o tempo de vida do processo.
- É registado um aviso sempre que um plugin é carregado por este caminho.
- As entradas em `legacy_libraries` também têm de aparecer em `load_libraries`.

Em caso de dúvida, recompile o plugin contra este repositório (host API v16) em vez de depender do caminho legado.

# Funcionalidades
- **Traduções de alta qualidade:** o Hachimi inclui funcionalidades de tradução avançadas que ajudam as traduções a parecerem mais naturais (formas plurais, números ordinais, etc.) e evitam introduzir falhas na interface. Também permite traduzir a maioria dos componentes do jogo; sem ser necessário aplicar patches manuais nos assets!

    Componentes suportados:
    - Texto da interface
    - master.mdb (nomes de habilidades, descrições de habilidades, etc.)
    - Histórias de corrida
    - História principal / diálogos do Home
    - Letras de músicas
    - Substituição de texturas
    - Substituição de atlas de sprites

    Além disso, o Hachimi não oferece funcionalidades de tradução para apenas um único idioma; foi concebido para ser totalmente configurável para qualquer idioma.

- **Configuração simples:** basta ligar e jogar. Toda a configuração é feita dentro do próprio jogo, sem ser necessária qualquer aplicação externa.
- **Atualização automática de traduções:** o atualizador de traduções integrado permite-lhe jogar normalmente enquanto está a atualizar, e recarrega-as dentro do jogo quando termina, sem ser necessário reiniciar!
- **Interface gráfica integrada:** inclui um editor de configurações para poder modificar as definições sem sequer sair do jogo!
- **Definições gráficas:** pode ajustar as definições gráficas do jogo para tirar o máximo partido das características do seu dispositivo, como o desbloqueio de FPS e o escalonamento de resolução.
- **Apenas Windows:** desenvolvido especificamente para a versão Windows (Steam) do jogo. **O HachimiRedux não suporta Android por opção** — concentra-se exclusivamente no cliente Windows, e não há planos para adicionar ou manter um build para Android.

# Instalação

A forma mais fácil de instalar o HachimiRedux é com o **instalador** da [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases): configura por si o mod principal e o plugin opcional Training Tracker, sem copiar ficheiros à mão nem editar JSON. Se preferir compilá-lo por si próprio, consulte [Compilar a partir do código-fonte](#compilar-a-partir-do-código-fonte-avançado).

O HachimiRedux é o mod principal (carregado como `cri_mana_vpx.dll`); o **Training Tracker** é um plugin DLL opcional carregado pelo mod principal. Ambos vêm do mesmo build.

A diretoria do jogo é a pasta de instalação do Steam, por exemplo
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`.

## Instalar com o instalador (recomendado)

1. Transfira o `hachimi_installer.exe` mais recente da [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases).
2. Execute-o. O instalador deteta automaticamente a diretoria do jogo no Steam; se não conseguir, selecione-a manualmente (o caminho predefinido está acima).
3. Escolha o seu idioma. Para ter o Training Tracker dentro do jogo, mantenha a caixa **"Install Training Tracker plugin"** marcada (ativada por predefinição).
4. Clique em **Install**. O instalador faz uma cópia de segurança do `cri_mana_vpx.dll` original, instala o mod e cria o `config.json` por si.
5. Inicie o jogo. Prima a tecla de menu — a predefinição é a tecla de **seta para a direita** — para abrir a interface dentro do jogo.

Para atualizar ou remover o HachimiRedux mais tarde, basta executar novamente o instalador (oferece uma opção de desinstalação).

## Compilar a partir do código-fonte (avançado)

Este repositório é um workspace do Cargo. A partir da raiz do repositório:

```sh
# Mod principal
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Plugin Training Tracker
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## Instalar o HachimiRedux (núcleo)

O jogo carrega o mod através da DLL de renderização `cri_mana_vpx.dll`.

1. Na diretoria do jogo, faça uma cópia de segurança do `cri_mana_vpx.dll` original como `cri_mana_vpx.dll.backup` (faça-o apenas uma vez — nunca substitua a cópia de segurança depois).
2. Copie `target/release/hachimi.dll` para a diretoria do jogo e mude o nome para `cri_mana_vpx.dll`.
3. Inicie o jogo. Prima a tecla de menu — a predefinição é a tecla de **seta para a direita** — para abrir a interface dentro do jogo. O ecrã de arranque mostra a tecla atual, e pode reatribuí-la a partir da interface gráfica dentro do jogo.

As definições do mod ficam em `config.json`, dentro da diretoria de dados do jogo, que é a **subpasta `hachimi` da diretoria do jogo** (por exemplo, `…\UmamusumePrettyDerby\hachimi\config.json`). É criada automaticamente pelo instalador / no primeiro arranque; tudo o resto é configurado a partir da interface gráfica dentro do jogo.

## Instalar o plugin Training Tracker

Os plugins são DLL nativas que o mod principal carrega no arranque a partir da raiz da diretoria do jogo.

1. Instale primeiro o núcleo do HachimiRedux (acima).
2. Copie `target/release/hachimi_training_tracker.dll` para a raiz da diretoria do jogo (a mesma pasta que `cri_mana_vpx.dll`). Nota: a DLL do plugin vai na **raiz** do jogo, enquanto o `config.json` fica na subpasta `hachimi`.
3. Adicione a DLL à lista `load_libraries` no `config.json` (`<game_dir>\hachimi\config.json`):

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. Inicie o jogo. O tracker aparece como uma página no separador Plugins e como um painel de sobreposição flutuante. Consulte [docs/plugin-sdk.md](docs/plugin-sdk.md) para perceber como os plugins funcionam.

## Implementação automatizada (Windows, a partir do código-fonte)

A partir da raiz do repositório, o script auxiliar compila e copia ambas as DLL para a diretoria do jogo:

```powershell
.\scripts\deploy-windows.ps1 -Build
```

Substitua a pasta do jogo se não estiver no caminho predefinido do Steam:

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

O script copia `hachimi.dll` → `cri_mana_vpx.dll` e a DLL do training tracker para a diretoria do jogo, e nunca modifica `cri_mana_vpx.dll.backup`.

# Dados de jogo alojados

O Training Tracker transfere os seus snapshots de dados do jogo (catálogos do GameTora, recursos do tracker, ícones de carreira) em tempo de execução a partir deste repositório, através de URLs raw do GitHub sob `main/data/…`. Os snapshots são regenerados pelo workflow diário [Data Refresh](.github/workflows/data_refresh.yml); a sequência manual está documentada em [docs/updating-game-data.md](docs/updating-game-data.md), com notas de manutenção em [MAINTENANCE.md](MAINTENANCE.md).

**Não renomeie este repositório, o ramo `main` nem o caminho `data/`** — as builds implementadas têm estes URLs fixos no código e deixariam de receber atualizações de dados.

# Resolução de problemas

## O jogo bloqueia ao iniciar / comporta-se de forma estranha

De longe, a causa mais comum é **empilhar vários mods de jogo ou injetores de DLL** na pasta do jogo. Cada um liga-se à renderização/runtime do jogo, e entram em conflito uns com os outros. O HachimiRedux avisa sobre isto dentro do jogo (uma notificação + o `hachimi.log`) e o instalador avisa antes de instalar, mas é você que tem de remover os restantes:

- Mantenha **apenas** o HachimiRedux: `cri_mana_vpx.dll` e quaisquer plugins compilados com o HachimiRedux (por exemplo, `hachimi_training_tracker.dll`).
- Remova da pasta do jogo outras sobreposições/injetores, como DLL de proxy-loader que não deveriam estar ali (`version.dll`, `winhttp.dll`, `dxgi.dll`, `d3d11.dll`, `dinput8.dll`, …) e sobreposições com nome (`horseACT.dll`, `heaven_overlay.dll`, …).
- **Apenas os plugins compilados com o HachimiRedux** pertencem ao `load_libraries`. Não adicione ali sobreposições de terceiros — não são plugins do HachimiRedux e serão rejeitadas (com um aviso dentro do jogo) ou podem bloquear o jogo.

## Onde fica cada coisa

- `cri_mana_vpx.dll` e as DLL de plugins: a diretoria **raiz** do jogo.
- `config.json` e outros dados do mod: a **subpasta `hachimi`** da diretoria do jogo (`<game_dir>\hachimi\config.json`).
- Registo do mod: `hachimi.log` na raiz do jogo (ative `enable_file_logging` no `config.json`).
- Registo do jogo: `%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`.

## Recolher diagnósticos

- Dentro do jogo: abra o menu (seta para a direita por predefinição) → **Config** → **Save diagnostics report**. Isto grava `hachimi_diagnostics.txt` na pasta do jogo.
- Instalador: execute `installer collect-logs` para reunir `config.json`, `hachimi.log` e um relatório de conflitos em `%TEMP%\hachimi_diagnostics`.

# Agradecimentos especiais

O HachimiRedux é um fork construído sobre o trabalho de:

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) — o projeto original em que se baseia. Se tiver interesse no projeto original, entre no [servidor do Discord dele](https://discord.gg/YjBgmuqqYr).
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) — o fork focado em Windows/Steam de onde o HachimiRedux dá continuidade.

Por sua vez, estes projetos serviram de base ao desenvolvimento do Hachimi; sem eles, o Hachimi nunca teria existido na sua forma atual:

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# Licença
[GNU GPLv3](LICENSE)
