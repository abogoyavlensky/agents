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

Then, if your skills directory is outside of the `Projects` directory you mounted above, 
add 

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

Or if you have your skills dir inside the `Projects` directory, you can just 
link it to the right place in the VM:

```bash
ln -s /workspace/Projects/agents/skills ~/.claude/skills
ln -s /workspace/Projects/agents/skills ~/.agents/skills
ln -s /workspace/Projects/agents/skills ~/.kiro/skills
```

## Set your git identity inside the VM

```shell
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
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

lmoc() {
  lm opencode "$@"
}

lmpi() {
  lm pi "$@"
}
```
