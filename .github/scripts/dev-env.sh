# shellcheck shell=bash
# Development environment for local builds and tests. Source it, do not run it:
#   source .github/scripts/dev-env.sh
# Points tests at an unpacked mpv/libmpv tree (see .spec/guide/mpv-install.guide.md).
# Override KINEKINO_LOCALDEPS or MEARI_FONT_CACHE before sourcing to use other paths.
KINEKINO_LOCALDEPS="${KINEKINO_LOCALDEPS:-<private-path>/.localdeps}"
export MEARI_FONT_CACHE="${MEARI_FONT_CACHE:-<private-path>/.kk-font-cache}"
if [ -d "$KINEKINO_LOCALDEPS" ]; then
  case ":$PATH:" in *":$KINEKINO_LOCALDEPS/bin:"*) ;; *) PATH="$KINEKINO_LOCALDEPS/bin:$PATH" ;; esac
  export PATH
  _kk_lib="$KINEKINO_LOCALDEPS/mpv/root/usr/lib/x86_64-linux-gnu"
  if [ -d "$_kk_lib" ]; then
    case ":${LD_LIBRARY_PATH:-}:" in
      *":$_kk_lib:"*) ;;
      *) export LD_LIBRARY_PATH="$_kk_lib:$_kk_lib/pipewire-0.3${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" ;;
    esac
    [ -e "$_kk_lib/libmpv.so.2" ] && export KINEKINO_LIBMPV="${KINEKINO_LIBMPV:-$_kk_lib/libmpv.so.2}"
  fi
  [ -x "$KINEKINO_LOCALDEPS/bin/mpv" ] && export KINEKINO_MPV="${KINEKINO_MPV:-$KINEKINO_LOCALDEPS/bin/mpv}"
  unset _kk_lib
fi
