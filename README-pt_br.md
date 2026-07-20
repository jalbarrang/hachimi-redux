<img align="left" width="80" height="80" src="apps/hachimi/assets/icon.png">

# HachimiRedux

[English](README.md) | [简体中文](README-zh_cn.md) | [繁體中文](README-zh_tw.md) | [Español (España)](README-es_es.md) | [Español (Latinoamérica)](README-es_419.md) | [Français](README-fr_fr.md) | Português (Brasil) | [Português (Portugal)](README-pt_pt.md)

Mod de aprimoramento e tradução do jogo para UM:PD. O HachimiRedux é um fork do Hachimi com um plugin de acompanhamento de treino integrado ao jogo e um SDK de plugins nativo reformulado.

<img height="400" src="apps/hachimi/assets/screenshot-2.png">

## Sumário

- [Por favor, não faça links para este repositório nem para o site do Hachimi](#️-por-favor-não-faça-links-para-este-repositório-nem-para-o-site-do-hachimi)
- [Incompatível com os plugins do Hachimi original](#️-incompatível-com-os-plugins-do-hachimi-original)
- [Recursos](#recursos)
- [Instalação](#instalação)
  - [Instalar com o instalador (recomendado)](#instalar-com-o-instalador-recomendado)
  - [Compilar a partir do código-fonte (avançado)](#compilar-a-partir-do-código-fonte-avançado)
- [Solução de problemas](#solução-de-problemas)
- [Agradecimentos especiais](#agradecimentos-especiais)
- [Licença](#licença)

# ⚠️ Por favor, não faça links para este repositório nem para o site do Hachimi
Entendemos que você queira ajudar as pessoas a instalar o Hachimi e a ter uma experiência melhor jogando. No entanto, este projeto vai, por natureza, contra os termos de serviço do jogo, e os desenvolvedores do jogo com certeza iriam querer que ele sumisse caso viessem a tomar conhecimento dele.

Compartilhar nos seus serviços de chat privados e por mensagens diretas tudo bem, mas pedimos humildemente que você evite compartilhar links para este projeto em sites públicos, ou para qualquer uma das ferramentas envolvidas.

Ou compartilhe mesmo assim e estrague tudo para a dúzia de usuários do Hachimi. A escolha é sua.

### Se você for compartilhar de qualquer forma
Faça o que precisar, mas pedimos respeitosamente que você tente rotular o jogo como "UM:PD" ou "The Honse Game" em vez do nome real do jogo, para evitar a indexação pelos mecanismos de busca.

# ⚠️ Incompatível com os plugins do Hachimi original
Este fork vem com sua própria API nativa de plugins (host API v16). **Plugins criados para o Hachimi original não são compatíveis com o HachimiRedux**, e o plugin de acompanhamento de treino distribuído aqui não vai carregar no Hachimi original. Prefira DLLs compiladas a partir deste repositório, e use-as em conjunto. Misturar builds pode falhar ao carregar ou travar o jogo.

## Compatibilidade com plugins legados (opcional, limitada)
Plugins sem manifesto e com ABI legada (por exemplo, os data-dumpers do Hachimi original) podem ser carregados por meio de um **caminho de compatibilidade opcional**. Adicione a DLL a uma lista de permissões `legacy_libraries` no `config.json`, além de `load_libraries`:

```json
{
  "windows": {
    "load_libraries": ["some_legacy_plugin.dll"],
    "legacy_libraries": ["some_legacy_plugin.dll"]
  }
}
```

Um plugin legado só precisa exportar `hachimi_init`; o host pula sua verificação habitual de manifesto/ABI e o carrega na confiança. Esse suporte é **limitado e sem garantias**:

- O plugin deve depender **apenas do prefixo estável da vtable** da API do host. Qualquer coisa além disso é comportamento indefinido e pode travar o jogo.
- O host **não consegue validar, rastrear nem descarregar** um plugin legado nem seus hooks de IL2CPP. A DLL permanece mapeada por toda a vida do processo.
- Um aviso é registrado sempre que um plugin é carregado por esse caminho.
- As entradas em `legacy_libraries` também precisam aparecer em `load_libraries`.

Na dúvida, recompile o plugin contra este repositório (host API v16) em vez de depender do caminho legado.

# Recursos
- **Traduções de alta qualidade:** o Hachimi vem com recursos de tradução avançados que ajudam as traduções a parecerem mais naturais (formas plurais, números ordinais, etc.) e evitam introduzir falhas na interface. Ele também permite traduzir a maioria dos componentes do jogo; sem precisar aplicar patches manuais nos assets!

    Componentes suportados:
    - Texto da interface
    - master.mdb (nomes de habilidades, descrições de habilidades, etc.)
    - Histórias de corrida
    - História principal / diálogos do Home
    - Letras de músicas
    - Substituição de texturas
    - Substituição de atlas de sprites

    Além disso, o Hachimi não oferece recursos de tradução para apenas um único idioma; ele foi projetado para ser totalmente configurável para qualquer idioma.

- **Configuração fácil:** é só plugar e jogar. Toda a configuração é feita dentro do próprio jogo, sem precisar de aplicativos externos.
- **Atualização automática de traduções:** o atualizador de traduções integrado permite que você jogue normalmente enquanto ele atualiza, e recarrega dentro do jogo quando termina, sem precisar reiniciar!
- **Interface gráfica integrada:** vem com um editor de configurações para você modificar os ajustes sem sequer sair do jogo!
- **Configurações gráficas:** você pode ajustar as configurações gráficas do jogo para aproveitar ao máximo as características do seu dispositivo, como o desbloqueio de FPS e o escalonamento de resolução.
- **Apenas Windows:** desenvolvido especificamente para a versão Windows (Steam) do jogo. **O HachimiRedux não dá suporte ao Android por escolha** — ele foca exclusivamente no cliente Windows, e não há planos de adicionar ou manter um build para Android.

# Instalação

A forma mais fácil de instalar o HachimiRedux é com o **instalador** da [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases): ele configura para você o mod principal e o plugin opcional Training Tracker, sem copiar arquivos na mão nem editar JSON. Se preferir compilar por conta própria, veja [Compilar a partir do código-fonte](#compilar-a-partir-do-código-fonte-avançado).

O HachimiRedux é o mod principal (carregado como `cri_mana_vpx.dll`); o **Training Tracker** é um plugin DLL opcional carregado pelo mod principal. Ambos vêm do mesmo build.

O diretório do jogo é a pasta de instalação do Steam, por exemplo
`C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby`.

## Instalar com o instalador (recomendado)

1. Baixe o `hachimi_installer.exe` mais recente na [página de Releases](https://github.com/jalbarrang/hachimi-redux/releases).
2. Execute-o. O instalador detecta automaticamente o diretório do jogo no Steam; se não conseguir, selecione-o manualmente (o caminho padrão está acima).
3. Escolha o seu idioma. Para ter o Training Tracker dentro do jogo, mantenha a caixa **"Install Training Tracker plugin"** marcada (ativada por padrão).
4. Clique em **Install**. O instalador faz um backup do `cri_mana_vpx.dll` original, instala o mod e cria o `config.json` para você.
5. Inicie o jogo. Pressione a tecla de menu — o padrão é a tecla de **seta para a direita** — para abrir a interface dentro do jogo.

Para atualizar ou remover o HachimiRedux depois, basta executar o instalador novamente (ele oferece uma opção de desinstalação).

## Compilar a partir do código-fonte (avançado)

Este repositório é um workspace do Cargo. A partir da raiz do repositório:

```sh
# Mod principal
cargo build --release -p hachimi                    # -> target/release/hachimi.dll
# Plugin Training Tracker
cargo build --release -p hachimi-training-tracker   # -> target/release/hachimi_training_tracker.dll
```

## Instalar o HachimiRedux (núcleo)

O jogo carrega o mod por meio da DLL de renderização `cri_mana_vpx.dll`.

1. No diretório do jogo, faça um backup do `cri_mana_vpx.dll` original como `cri_mana_vpx.dll.backup` (faça isso uma única vez — nunca sobrescreva o backup depois).
2. Copie `target/release/hachimi.dll` para o diretório do jogo e renomeie para `cri_mana_vpx.dll`.
3. Inicie o jogo. Pressione a tecla de menu — o padrão é a tecla de **seta para a direita** — para abrir a interface dentro do jogo. A tela de abertura mostra a tecla atual, e você pode reatribuí-la pela interface gráfica dentro do jogo.

As configurações do mod ficam em `config.json`, dentro do diretório de dados do jogo, que é a **subpasta `hachimi` do diretório do jogo** (por exemplo, `…\UmamusumePrettyDerby\hachimi\config.json`). Ela é criada automaticamente pelo instalador / na primeira execução; todo o resto é configurado pela interface gráfica dentro do jogo.

## Instalar o plugin Training Tracker

Plugins são DLLs nativas que o mod principal carrega na inicialização a partir da raiz do diretório do jogo.

1. Instale primeiro o núcleo do HachimiRedux (acima).
2. Copie `target/release/hachimi_training_tracker.dll` para a raiz do diretório do jogo (a mesma pasta que `cri_mana_vpx.dll`). Observação: a DLL do plugin vai na **raiz** do jogo, enquanto o `config.json` fica na subpasta `hachimi`.
3. Adicione a DLL à lista `load_libraries` no `config.json` (`<game_dir>\hachimi\config.json`):

   ```json
   {
     "windows": {
       "load_libraries": ["hachimi_training_tracker.dll"]
     }
   }
   ```
4. Inicie o jogo. O tracker aparece como uma página na aba Plugins e como um painel de sobreposição flutuante. Veja [docs/plugin-sdk.md](docs/plugin-sdk.md) para entender como os plugins funcionam.

## Deploy automatizado (Windows, a partir do código-fonte)

A partir da raiz do repositório, o script auxiliar compila e copia ambas as DLLs para o diretório do jogo:

```powershell
.\scripts\deploy-windows.ps1 -Build
```

Substitua a pasta do jogo se ela não estiver no caminho padrão do Steam:

```powershell
$env:HACHIMI_GAME_DIR = "D:\path\to\UmamusumePrettyDerby"
.\scripts\deploy-windows.ps1 -Build
```

O script copia `hachimi.dll` → `cri_mana_vpx.dll` e a DLL do training tracker para o diretório do jogo, e nunca modifica `cri_mana_vpx.dll.backup`.

# Dados do jogo hospedados

O Training Tracker baixa seus snapshots de dados do jogo (catálogos do GameTora, recursos do tracker, ícones de carreira) em tempo de execução a partir deste repositório, via URLs raw do GitHub sob `main/data/…`. Os snapshots são regenerados pelo workflow diário [Data Refresh](.github/workflows/data_refresh.yml); a sequência manual está documentada em [docs/updating-game-data.md](docs/updating-game-data.md), com notas de manutenção em [MAINTENANCE.md](MAINTENANCE.md).

**Não renomeie este repositório, a branch `main` nem o caminho `data/`** — as builds implantadas têm essas URLs fixas no código e deixariam de receber atualizações de dados.

# Solução de problemas

## O jogo trava ao iniciar / se comporta de forma estranha

De longe, a causa mais comum é **empilhar vários mods de jogo ou injetores de DLL** na pasta do jogo. Cada um se prende à renderização/runtime do jogo, e eles brigam entre si. O HachimiRedux avisa sobre isso dentro do jogo (uma notificação + o `hachimi.log`) e o instalador avisa antes de instalar, mas você mesmo precisa remover os outros:

- Mantenha **apenas** o HachimiRedux: `cri_mana_vpx.dll` e quaisquer plugins compilados com o HachimiRedux (por exemplo, `hachimi_training_tracker.dll`).
- Remova da pasta do jogo outras sobreposições/injetores, como DLLs de proxy-loader que não deveriam estar ali (`version.dll`, `winhttp.dll`, `dxgi.dll`, `d3d11.dll`, `dinput8.dll`, …) e sobreposições nomeadas (`horseACT.dll`, `heaven_overlay.dll`, …).
- **Apenas plugins compilados com o HachimiRedux** pertencem ao `load_libraries`. Não adicione ali sobreposições de terceiros — elas não são plugins do HachimiRedux e serão rejeitadas (com um aviso dentro do jogo) ou podem travar o jogo.

## Onde fica cada coisa

- `cri_mana_vpx.dll` e as DLLs de plugins: o diretório **raiz** do jogo.
- `config.json` e outros dados do mod: a **subpasta `hachimi`** do diretório do jogo (`<game_dir>\hachimi\config.json`).
- Log do mod: `hachimi.log` na raiz do jogo (ative `enable_file_logging` no `config.json`).
- Log do jogo: `%USERPROFILE%\AppData\LocalLow\Cygames\Umamusume\Player.log`.

## Coletar diagnósticos

- Dentro do jogo: abra o menu (seta para a direita por padrão) → **Config** → **Save diagnostics report**. Isso grava `hachimi_diagnostics.txt` na pasta do jogo.
- Instalador: execute `installer collect-logs` para reunir `config.json`, `hachimi.log` e um relatório de conflitos em `%TEMP%\hachimi_diagnostics`.

# Agradecimentos especiais

O HachimiRedux é um fork construído sobre o trabalho de:

- [Hachimi](https://github.com/Hachimi-Hachimi/Hachimi) — o projeto original em que se baseia. Se você tiver interesse no projeto original, entre no [servidor do Discord dele](https://discord.gg/YjBgmuqqYr).
- [Hachimi Edge](https://github.com/kairusds/Hachimi-Edge) — o fork focado em Windows/Steam de onde o HachimiRedux dá continuidade.

Por sua vez, esses projetos serviram de base para o desenvolvimento do Hachimi; sem eles, o Hachimi nunca teria existido na sua forma atual:

- [Trainers' Legend G](https://github.com/MinamiChiwa/Trainers-Legend-G)
- [umamusume-localify-android](https://github.com/Kimjio/umamusume-localify-android)
- [umamusume-localify](https://github.com/GEEKiDoS/umamusume-localify)
- [Carotenify](https://github.com/KevinVG207/Uma-Carotenify)
- [umamusu-translate](https://github.com/noccu/umamusu-translate)
- [frida-il2cpp-bridge](https://github.com/vfsfitvnm/frida-il2cpp-bridge)

# Licença
[GNU GPLv3](LICENSE)
