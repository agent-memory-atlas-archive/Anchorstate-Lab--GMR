# Review brief: GMR future architecture (2026-09-08)

Repository: /Users/zongming/Desktop/gmr (branch design/ontology-v1). Read-only review. Do NOT modify repository files. Write your report to the path given in your prompt.

## The owner's question (verbatim, Chinese)

结合knowledge graph, ontology, GMR进行深度思考, 目标生态环境是怎么样的? 我们应该怎么做? 目前已经发现的问题如下:

1. 针对某一项任务而言, 一个模型能否完成这个任务, 输入给模型的所有信息中存在一个最小充分信息集 (MSIS), 它需要具备信息的完备性和最小性. 不完备 → 模型靠假设/猜测补全 → 幻觉. 信息过大 → 上下文被占用, 注意力稀释, 任务中心漂移.
2. GMR 的作用就是在事实, AI推论, 锚点之间构建连接关系, 最终形成复杂网络. 这是基础设施; 设计好之后很多功能会自发涌现. 这个网络是 knowledge graph 的进阶版本: 沿一个点的边找就能找到完备信息; 若每条信息原子级无噪音, 子图不会太大. 不能严格证明最小/完备, 但比单点更全, 比全量读取更小, 比 AI 随意 tool 搜索更有效. 每个 AI 不断分析更新节点和边 → 动态平衡 → 收敛到最佳状态.
3. 实测证据: 项目中模块 A 和 B 有业务逻辑关系但无依赖关系; 只靠依赖图深度<5 找不到, 只能枚举, 浪费 token. GMR 中事实层无直接联系, 但记忆之间有边, 很容易找到. 这是 GMR 有效性的证据.
4. 现状问题: 事实层都是孤立节点 (代码有依赖关系, 非代码信息也有逻辑因果关系, 事实层没做). fact / anchor / memory 三层都应该是独立的图, 三层之间节点又相连, 空间复杂度高, 当前代码偷懒了. fact 节点是否就应该是 knowledge graph? anchor 是状态机图? memory 是 ontology? 这样的设计是否合适?
5. 存储和输出协议非常差: Agent 存记忆不是原子化的, 噪音多 (没有规范 + skill 没有正确引导). 输出 JSON 时信封 token 比有效信息 token 更高, 直接让 MSIS 变成伪命题. 所有动词使用中都有噪音: 每一步怎么用, 应该吐出什么, 怎么简化? 应重新设计记忆文本的协议规范, 结合 ontology 思考.

要求: 先质疑现在的系统, 不要给现在的系统找存在理由, 而是去找真正的未来架构. 分析清楚: 未来生态系统是什么样, 我们要做的是什么, GMR 的定位在哪, 目前存在哪些问题, 设计到实施上有什么可以修改.

## Owner-accepted target direction (2026-09-07, from the owner's memory; treat as the current hypothesis to stress-test, not as scripture)

- Net (fact layer) is the product: a KG of entities with ontology-defined keys, typed relations (calls, imports, produces/consumes Format, governed_by, decided_by, must_change_with...), properties and task Claims, each with provenance to a reading. Only source is the task-doing AI, which reads reality with any tool and writes what fills a declared slot at task end (add/update/supersede/retire). Completeness grows with tasks.
- Ontology (memory layer) is the agent's interface and the memory protocol: object types, relation types, action signatures (= closure rules = MSIS definition), constraints, Decision/Policy with rationale. Human-designed, versioned, rarely changed. Write rule: a memory is an instance filling a declared slot of a declared entity with a declared source; information with no slot is not written.
- Anchors = staleness only: probes compute fact addresses; when a reading changes, facts resting on it are marked stale. Nothing regenerates automatically; the next agent that touches a stale item fixes it.
- Convergence has one mechanism: agents update what they read as part of finishing tasks. No workers, judge loops, validation gates. Metric: stale marks cleared over tasks.
- Delivery: locate (hybrid retrieval) to an entity, then enter(action, entity) traverses by action signature and serializes a compact subgraph; never waits on a model. Verbs: locate / enter / read / write / check.

## Files to read (in this order; all paths relative to the repo)

1. docs/ontology/coding-v1.yaml — the draft ontology (183 lines). Central object of review.
2. docs/ontology/migration-trial.md — 16 notes mapped onto v1 slots; ~half of note text fills no slot.
3. docs/problems.md — the owner's own evidence-based problem list (Chinese). Sections 一–七 are measured problems in the CURRENT system: 74% of delivered memories are not "about" the coordinate; `grounded` field inverted; links table = frozen wikilinks all typed `cites`; settled ≠ correct; positioning never anchored to code; envelope not in contract; full-repo cost per key; three SKILL.md copies. Section 零 states "GMR does not compute MSIS" (objective depends on task; task unknown at selection time).
4. .claude/skills/gmr/SKILL.md — the current agent-facing protocol (28 KB of prose). CLAUDE.md is another 14 KB every agent loads.
5. memories/README.md — the current memory file format. memories/three-layers.md and memories/gmr-not-entailment.md — the two positioning notes (fact/memory/inference; structure-not-entailment).
6. docs/ARCHITECTURE.md §1–§3.10, §4.5–4.6, §8, §10 — current design SSOT (86 KB; read selectively).
7. crates/ layout: gmr-core, gmr-expr, gmr-budget, gmr-probe, gmr-content, gmr-store, gmr-runtime; console/cli; batteries/; packs/coding. Use `codegraph explore "<symbol>"` or read files if you need code facts.

## Measurements taken today (release binary gmr 0.6.3, this repo)

- Corpus: 196 notes, 640 KB total, p50 2.5 KB, p90 6.5 KB, max 14.2 KB. 434 [[wikilinks]] in prose; only 3 notes declare typed `links:` (all `rests-on`). Store links table: 381 `cites` + 7 `rests-on`. 847 anchors open. 38 notes are `about:` one file (crates/gmr-runtime/src/read.rs).
- `gmr read crates/gmr-runtime/src/memory.rs#carry_linked --json`: 18,564 bytes total. Breakdown: `anchor` (declaration: probe recipe, rule table, terminal) 9,428 B = 51%; `memories` 3,624 B of which memory TEXT 3,029 B = 16% of the whole; per-memory `grounding` (before/after diff) 3,034 B; `facts` 888 B; `state` 1,354 B (baseline+now each 547 B: name, file, form, sig-string 331 B, surface, body hash, after). `--lean` still 15,639 B with zero memory text. So envelope ≈ 5× payload for one coordinate.
- `gmr status <key> --json`: 1,572 B. `gmr check --json` (nothing moved): 219 B. Human `gmr read <key>` output: 3 lines.
- Every agent session loads SKILL.md (28 KB) + CLAUDE.md (14 KB) ≈ 42 KB ≈ 10k tokens before any task-specific information.
- (From problems.md, earlier run at 659 anchors) full-repo `gmr read --json` delivers 2,328 records, 10.3 MB; 1,713 of them (74%) reached only via links; per-key `status`/`check` cost 3.6×/5.8× of `read` because they scan all notes.

## Ground rules for your report

- Challenge, do not defend. If something in the current system or the target direction is wrong, say so and say why with evidence (file:line or a measurement). If something is right, one sentence, move on.
- Be concrete: names of types, slots, relations, verbs, byte budgets, phases. No generic advice ("add typed edges") that the owner already has.
- Distinguish: (a) measured fact, (b) inference from code, (c) opinion.
- Length: ≤ 1,800 words in the report file. Then return a ≤ 300-word summary as your final message with your 3–5 sharpest claims.
- Write in English. The owner reads both; English keeps terminology stable.
