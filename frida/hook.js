const getMainModule = (version) => {
    if (version >= 13331) {
        return Process.findModuleByName("flue.dll");
    }
    return Process.findModuleByName("WeChatAppEx.exe");
};

const patchCDPFilter = (base, config) => {
    // xref: SendToClientFilter OR devtools_message_filter_applet_webview.cc
    const offset = config.CDPFilterHookOffset;
    let patchCount = 0;
    Interceptor.attach(base.add(offset), {
        onEnter(args) {
            this.inputValue = args[0];
        },
        onLeave(retval) {
            const inputValue = this.inputValue.readPointer();
            if (inputValue.isNull() || inputValue.add(8).isNull()) {
                // there's a chance the value could be null
                // return here to avoid crash
                return;
            }

            if (inputValue.add(8).readU32() == 6) {
                inputValue.add(8).writeU32(0x0);
                patchCount += 1;
                if (patchCount <= 5 || patchCount % 50 === 0) {
                    send(
                        `[patch] CDP filter patched type 6 -> 0, count=${patchCount}`,
                    );
                }
            }
        },
    });
};

const tryReadPointer = (ptr) => {
    try {
        if (ptr.isNull()) {
            return null;
        }
        const value = ptr.readPointer();
        return value.isNull() ? null : value;
    } catch (e) {
        return null;
    }
};

const tryReadInt = (ptr) => {
    try {
        if (ptr.isNull()) {
            return null;
        }
        return ptr.readInt();
    } catch (e) {
        return null;
    }
};

const sceneOffsetCandidates = [
    [56, 1208, 8, 1160, 16, 488],
    [56, 1272, 8, 1224, 16, 488],
    [56, 1280, 8, 1232, 16, 488],
    [56, 1360, 8, 1312, 16, 488],
    [56, 1416, 8, 1360, 16, 488],
    [56, 1408, 8, 1352, 16, 488],
    [56, 1416, 8, 1352, 16, 488],
    [56, 1376, 8, 1312, 16, 456],
    [64, 1408, 8, 1344, 16, 456],
    [64, 1472, 8, 1408, 16, 456],
    [64, 1480, 8, 1416, 16, 456],
];

const probeLoggedPointers = new Set();

const resolveScenePtr = (a1, sceneOffsets) => {
    const first = tryReadPointer(a1.add(sceneOffsets[0]));
    if (first === null) {
        return { ok: false, stage: "first pointer is invalid" };
    }

    const miniappConfigPtr = tryReadPointer(first.add(sceneOffsets[1]));
    if (miniappConfigPtr === null) {
        return { ok: false, stage: "config pointer is invalid" };
    }

    const second = tryReadPointer(miniappConfigPtr.add(sceneOffsets[2]));
    if (second === null) {
        return { ok: false, stage: "second pointer is invalid" };
    }

    const third = tryReadPointer(second.add(sceneOffsets[3]));
    if (third === null) {
        return { ok: false, stage: "third pointer is invalid" };
    }

    const fourth = tryReadPointer(third.add(sceneOffsets[4]));
    if (fourth === null) {
        return { ok: false, stage: "fourth pointer is invalid" };
    }

    const scenePtr = fourth.add(sceneOffsets[5]);
    const sceneValue = tryReadInt(scenePtr);
    if (sceneValue === null) {
        return { ok: false, stage: "scene value is unreadable" };
    }

    return { ok: true, scenePtr, sceneValue };
};

const probeSceneOffsets = (a1) => {
    const key = a1.toString();
    if (probeLoggedPointers.has(key)) {
        return;
    }
    probeLoggedPointers.add(key);

    for (const candidate of sceneOffsetCandidates) {
        const result = resolveScenePtr(a1, candidate);
        if (!result.ok) {
            continue;
        }

        if (result.sceneValue >= 0 && result.sceneValue <= 5000) {
            send(
                `[probe] candidate ${JSON.stringify(candidate)} -> scene ${result.sceneValue}`,
            );
        }
    }
};

const hookOnLoadScene = (a1, sceneOffsets) => {
    const result = resolveScenePtr(a1, sceneOffsets);
    if (!result.ok) {
        send(`[hook] scene chain aborted: ${result.stage}`);
        probeSceneOffsets(a1);
        return;
    }

    const miniappScenePtr = result.scenePtr;
    const sceneValue = result.sceneValue;
    send(`[hook] scene: ${sceneValue}`);

    // 1000: from issue #83 <-- will crash the process
    // 1007: from issue #80
    // 1008: from issue #53
    // 1027: from issue #78
    // 1035: from issue #78
    // 1053: from issue #25
    // 1074: from issue #32
    // 1145: from search
    // 1178: from phone (issue #117)
    // 1256: from recent
    // 1260: from frequently used
    // 1302: from services
    // 1308: minigame?
    const sceneNumberArray = [
        1005, 1007, 1008, 1027, 1035, 1053, 1074, 1145, 1178, 1256, 1260, 1302,
        1308,
    ];
    if (!sceneNumberArray.includes(sceneValue)) {
        return;
    }
    send("[hook] hook scene condition -> 1101");
    try {
        miniappScenePtr.writeInt(1101);
    } catch (e) {
        send(`[hook] failed to write scene value: ${e}`);
    }

    // TODO: customize debugging endpoint
    // const websocketServerStringPtr = passArgs.add(8).readPointer().add(520);
    // VERBOSE && console.log("[hook] hook websocket server, original: ", websocketServerStringPtr.readUtf8String());
    // websocketServerStringPtr.writeUtf8String("ws://127.0.0.1:8189/");
};

const patchOnLoadStart = (base, config) => {
    // xref: AppletIndexContainer::OnLoadStart
    Interceptor.attach(base.add(config.LoadStartHookOffset), {
        onEnter(args) {
            send(
                `[inteceptor] AppletIndexContainer::OnLoadStart onEnter, ` +
                    `indexContainer.this: ${this.context.rcx}`,
            );
            // write dl to 0x1
            if ((this.context.rdx & 0xff) !== 1) {
                this.context.rdx = (this.context.rdx & ~0xff) | 0x1;
            }
            // handle onLoad scene
            try {
                hookOnLoadScene(this.context.rcx, config.SceneOffsets);
            } catch (e) {
                send(`[hook] OnLoadStart scene hook exception: ${e}`);
            }
        },
        onLeave(retval) {
            // do nothing
        },
    });
};

const parseConfig = () => {
    const rawConfig = `@@CONFIG@@`;
    if (rawConfig.includes("@@")) {
        // test addresses
        return {
            Version: 18955,
            LoadStartHookOffset: "0x25B52C0",
            CDPFilterHookOffset: "0x30248B0",
            SceneOffsets: [1408, 1344, 488],
        };
    }
    return JSON.parse(rawConfig);
};

const main = () => {
    const config = parseConfig();
    const mainModule = getMainModule(config.Version);
    patchOnLoadStart(mainModule.base, config);
    patchCDPFilter(mainModule.base, config);
};

main();
