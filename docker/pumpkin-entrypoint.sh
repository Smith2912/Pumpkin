#!/usr/bin/env sh
set -eu

data_root=/pumpkin
server_id=pumpkin
server_root="${data_root}/${server_id}"
panel_root="${data_root}/.pufferpanel"
config_file="${panel_root}/config.json"
layout_marker="${panel_root}/layout-v1.complete"
bootstrap_marker="${panel_root}/bootstrap-v1.complete"
bootstrap_cookie="${panel_root}/bootstrap-cookie.txt"
server_definition="${data_root}/${server_id}.json"
server_import="${panel_root}/server-import-v1"

run_as_pumpkin() {
    if [ "$(id -u)" -eq 0 ]; then
        gosu pumpkin "$@"
    else
        "$@"
    fi
}

mkdir -p "${panel_root}"

# Migrate the original volume layout into PufferPanel's per-server folder.
# Every moved top-level entry is recorded so the migration can be reversed.
if [ ! -f "${layout_marker}" ]; then
    find "${data_root}" -mindepth 1 -maxdepth 1 \
        ! -name "${server_id}" \
        ! -name '.pufferpanel' \
        -printf '%f\n' > "${panel_root}/layout-v1.manifest"

    mkdir -p "${server_import}"

    while IFS= read -r entry; do
        [ -n "${entry}" ] || continue
        mv "${data_root}/${entry}" "${server_import}/${entry}"
    done < "${panel_root}/layout-v1.manifest"

    touch "${layout_marker}"
fi

# Recover a first boot interrupted after the legacy layout was moved but before
# PufferPanel could create its own server directory and definition.
if [ ! -f "${bootstrap_marker}" ] && [ ! -f "${server_definition}" ] \
    && [ -d "${server_root}" ] && [ ! -d "${server_import}" ]; then
    mv "${server_root}" "${server_import}"
fi

if [ ! -f "${config_file}" ]; then
    cp /opt/pumpkin/pufferpanel/config.json "${config_file}"
fi

mkdir -p "${panel_root}/logs" "${panel_root}/backups" \
    "${panel_root}/cache" "${panel_root}/binaries"

if [ "$(id -u)" -eq 0 ] && [ ! -f "${panel_root}/ownership-v1.complete" ]; then
    chown -R 2613:2613 "${data_root}"
    touch "${panel_root}/ownership-v1.complete"
    chown 2613:2613 "${panel_root}/ownership-v1.complete"
fi

export PUFFER_CONFIG="${config_file}"

stop_bootstrap_panel() {
    if [ -n "${bootstrap_pid:-}" ] && kill -0 "${bootstrap_pid}" 2>/dev/null; then
        kill -TERM "${bootstrap_pid}" 2>/dev/null || true
        wait "${bootstrap_pid}" 2>/dev/null || true
    fi
}

if [ ! -f "${bootstrap_marker}" ]; then
    : "${PUFFER_ADMIN_PASSWORD:?PUFFER_ADMIN_PASSWORD is required for the first PufferPanel boot}"
    admin_email="${PUFFER_ADMIN_EMAIL:-admin@pumpkin.local}"

    run_as_pumpkin /usr/local/bin/pufferpanel run &
    bootstrap_pid=$!
    trap stop_bootstrap_panel EXIT INT TERM

    attempts=0
    until curl -fsS "http://127.0.0.1:8080/api/config" >/dev/null 2>&1; do
        attempts=$((attempts + 1))
        if [ "${attempts}" -ge 60 ] || ! kill -0 "${bootstrap_pid}" 2>/dev/null; then
            echo "PufferPanel did not become ready for first-boot configuration" >&2
            exit 1
        fi
        sleep 1
    done

    login_payload=$(printf '{"email":"%s","password":"%s"}' \
        "${admin_email}" "${PUFFER_ADMIN_PASSWORD}")

    if ! curl -fsS -c "${bootstrap_cookie}" \
        -H 'Content-Type: application/json' \
        -d "${login_payload}" \
        "http://127.0.0.1:8080/auth/login" >/dev/null 2>&1; then
        run_as_pumpkin /usr/local/bin/pufferpanel user add \
            --name admin \
            --email "${admin_email}" \
            --password "${PUFFER_ADMIN_PASSWORD}" \
            --admin

        curl -fsS -c "${bootstrap_cookie}" \
            -H 'Content-Type: application/json' \
            -d "${login_payload}" \
            "http://127.0.0.1:8080/auth/login" >/dev/null
    fi

    server_status=$(curl -sS -o /dev/null -w '%{http_code}' \
        -b "${bootstrap_cookie}" \
        "http://127.0.0.1:8080/api/servers/${server_id}")

    case "${server_status}" in
        200)
            if [ ! -f "${server_definition}" ]; then
                curl -fsS -b "${bootstrap_cookie}" \
                    -H 'Content-Type: application/json' \
                    -X PUT \
                    --data-binary @/opt/pumpkin/pufferpanel/pumpkin.json \
                    "http://127.0.0.1:8080/daemon/server/${server_id}" >/dev/null
            fi
            ;;
        404)
            curl -fsS -b "${bootstrap_cookie}" \
                -H 'Content-Type: application/json' \
                -X PUT \
                --data-binary @/opt/pumpkin/pufferpanel/pumpkin.json \
                "http://127.0.0.1:8080/api/servers/${server_id}" >/dev/null
            ;;
        *)
            echo "Unexpected PufferPanel server lookup status: ${server_status}" >&2
            exit 1
            ;;
    esac

    stop_bootstrap_panel
    rm -f "${bootstrap_cookie}"

    if [ -d "${server_import}" ]; then
        find "${server_import}" -mindepth 1 -maxdepth 1 \
            -exec mv -t "${server_root}" -- {} +
        rmdir "${server_import}"
    fi

    touch "${bootstrap_marker}"
    trap - EXIT INT TERM
fi

# The Railway volume hides image files mounted at /pumpkin. Install the native
# compatibility plugin into the managed server directory on every boot.
mkdir -p "${server_root}/plugins"
cp /opt/pumpkin/plugins/libpatchbukkit.so "${server_root}/plugins/libpatchbukkit.so"

if [ "$(id -u)" -eq 0 ]; then
    exec gosu pumpkin /usr/local/bin/pufferpanel run
else
    exec /usr/local/bin/pufferpanel run
fi
