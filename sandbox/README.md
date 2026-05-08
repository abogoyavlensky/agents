# Agent Sandbox using Lima VM

## Install Lima

```bash
brew install lima
```

## Create the agent sandbox

```bash
limactl create --name sandbox ./agent.yaml
```

## Mount projects dir

Mount the directory you want the VM to see and start the VM. You only
need to do this once; you can change the mount later.

```bash
cd ~/Projects   # or any directory you want to expose
limactl edit sandbox --mount-only .:w --start
```


## Share skills dir with agents

```bash
limactl stop sandbox
limactl edit sandbox
```

TODO: move agents repo out of `Projects` dir to not overlap mounted dirs inside the VM.

```bash
mounts:
...
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.claude/skills"
  writable: true
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.agents/skills"
  writable: true
- location: "/Users/andrew/Projects/agents/skills"
  mountPoint: "/home/agent.guest/.kiro/skills"
  writable: true
```

## Aliases

```bash
lm() {
  LIMA_SHELLENV_BLOCK=* LIMA_SHELLENV_ALLOW=GH_TOKEN limactl shell --preserve-env $LIMA_DEFAULT_VM -- "$@"
}

# Open a shell in the VM with GH_TOKEN forwarded: `lmsh`
lmsh() {
  LIMA_SHELLENV_BLOCK=* LIMA_SHELLENV_ALLOW=GH_TOKEN limactl shell --preserve-env $LIMA_DEFAULT_VM
}

lmcc() {
  lm claude --dangerously-skip-permissions "$@"
}

lmcx() {
  lm codex --yolo "$@"
}

lmcop() {
  lm copilot --yolo "$@"
}

lmkiro() {
  lm kiro-cli chat --trust-all-tools "$@"
}
```
