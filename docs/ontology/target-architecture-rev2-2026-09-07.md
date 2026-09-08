# GMR 目标架构（2026-09-07）

本文是当前设计。它只写设计是什么，不写论证过程和曾经的版本；那些属于会话记录，不属于后面的 agent 要读的东西。

---

## 1. 目标

一个 agent 接到任务，从一个实体进入，拿到关于这项任务的完备且小的信息集；世界变了它仍正确；用得越多越清楚。为此建一张网，AI 在做任务时建它、用它、改它；人只设计本体和写不可推导的决定。这是基础设施：设计好之后功能从使用中挣出来，不单独开发。

---

## 2. 三层

| 层 | 是什么 | 谁写 | 工业界对应 |
|---|---|---|---|
| fact 层 = 网（KG，ABox） | 实体 + 类型化关系 + 属性 + 结论，全部带依据 | AI，在任务中从读数得出 | 代码 KG（contains / imports / calls + 语义关系 + 文本）、Graphiti 的双时态边 |
| memory 层 = 本体（TBox） | 对象类型、关系类型、动作签名、约束、决定与理由 | 人设计、人批准；AI 可提案 | Palantir Ontology 的 nouns + verbs、OaK 的 schema + functions |
| anchor 层 = 时效 | 探针读现实，算 fact address，读数变了把依据它的事实标 stale | 机械 | Graphiti 靠 LLM 判 t_invalid，这里靠观测 |

三层不是三张图，是一张图的三个方面：本体是它的 schema，网是它的实例，锚是每个实例通向现实的线。

---

## 3. 本体（memory 层）

本体是 agent 的接口：它规定世界里有什么、怎么相连、能做什么动作、每个动作要读什么。小、稳定、人维护、有版本。以 coding 域为例：

**对象类型**：Module · Function · Type · Format · Endpoint · Table · Config · Test · Policy · Decision · Incident · Owner · Claim（任务结论）。

**关系类型**（有向，每种一句定义）：calls · imports · implements · contains · produces(Format) · consumes(Format) · uses(Config) · verified-by(Test) · governed-by(Policy) · decided-by(Decision) · must-change-with · supersedes · contradicts · rests-on(读数或其他事实)。calls 和 consumes 同级，本体不区分句法与语义。

**动作签名**（= 闭包规则，即 MSIS 的定义）：

```
modify(Function f)  读 f · f.calls⁻¹ · f.produces/consumes 的 Format 及其另一端 · f.uses 的 Config
                    · governed-by(f 或其 Module) 的 Policy · decided-by 的 Decision · verified-by 的 Test
                    · must-change-with 的对端 · 未关闭的 Incident · 站在 f 上的 Claim
extend(Module m)    读 m.contains 的现有成员形态 · m 的 Policy · 注册点（must-change-with）
explain(x)          读 x 的属性 · x.rests-on 一到两跳 · Decision 及理由
verify(Claim c)     读 c.rests-on 的读数与当前读数对比 · 与 c contradicts 的事实
```

**约束与决定**：能机械表达的写成约束（Format:ledger_line.amount 是整数分；pricing 不 imports report）；不能机械表达的写成 Decision，理由是它的属性，governs 它约束的实体。这就是"结合 ontology 重设计记忆协议"：memories/ 里 197 篇散文中可建模的部分变成类型、关系、约束、决定；散文只剩理由字段。

本体的变化是设计行为，罕见，由人做。AI 在任务中发现本体表达不了的东西，写成 Claim 提案。

### 3.1 记忆协议：本体决定什么能被记住

记忆不是文本，是本体类型的实例，填进声明的槽位。本体因此规定了五件事：能记什么（类型）、关于什么（关系的两端类型）、什么形状（属性）、凭什么（source 的种类）、什么不是记忆。

**写入判据只有一条：这条信息填的是哪个实体的哪个槽位？答不出来就不写。**

用一个非代码的例子说明它挡住什么。用户说"想吃苹果"，本体里有 Person、Food，关系 wants(Person → Food) 带属性 {at, source}，关系 contains(Fridge → Food)，而 Apple is-a Fruit 是本体自身的知识。

| 信息 | 填哪个槽位 | 处置 |
|---|---|---|
| 用户想吃苹果 | Person:zongming wants Food:Apple，at，source = 用户说的 | 写，且只写这一条 |
| 冰箱里有苹果 | Fridge contains Apple，是冰箱的状态 | 是这次任务读到的现实，用于满足意图；不挂在用户的意图上；除非有任务需要它，不写 |
| 用户不吃梨、不吃香蕉 | dislikes 关系要求 source 是用户说的或观察到的 | 没有来源，不能写成关于用户的事实；要留的话只能是本任务的 Claim，随任务退役 |
| 苹果是水果 | 本体的类型层级 | 本体已有，实例层没有这个槽位，写不进去 |

四条在事实层面都对，只有第一条是关于这个人的记忆。噪音不是假信息，是**没有槽位的真信息**。

代码域同理。Decision 与 Policy 的槽位是：about（实体）、claim（一句）、rationale（为什么）、source（事故、PR、所有者）、valid_from。不属于任何槽位、因此不写的：这条是怎么被发现的；以前错在哪、曾经是什么样；代码本身做什么（那是读数，读现实就有）；别的节点说了什么（那是一条边，不是正文）；某次测量的数字（那是读数的快照）。现有 197 篇笔记里约三分之一含这类内容，迁移时只取槽位，其余丢弃。

**历史不是记忆。** 一个节点只有现在的值；怎么到这里的，在 journal 里可以重放，但不在节点上。后面的 agent 读到的是状态，不是路径，所以不必为前人的弯路付费。

**读取侧由此不需要过滤。** 交付的是槽位的值，不是正文，节点内部没有地方放噪音；只有 Decision.rationale 是自由文本，而它只在 explain 与 verify 的动作签名里被读。

**结论与记忆分开。** agent 在任务里的推理、假设、中间判断是 Claim，带 task 与 rests-on，任务的依据过时即退役。它们从不进入实体的槽位；进入槽位的只有 agent 从读数确认的关系和属性。这就是"想吃苹果"不会长出"不吃梨"的机制：推断没有槽位。

---

## 4. 网（fact 层）

**节点**：实体，键由本体定义（Function 的键 = 路径 + 名字 + 签名形状，Format 的键 = 名字，Decision 的键 = id）。属性是原子的、带依据的值：Function.purpose = "…" rests-on 读数 X。

**边**：(类型, 起点, 终点) 唯一，带作者、依据（fact address 或其他事实）、理由、valid_from、superseded_by。两个 agent 写同一条边是同一条边；写不同的目标是两条边，都交付，下一个读现实的 agent 改掉一条。

**结论**：Claim 实体，属性是那句话，rests-on 它依据的读数与事实，author = 任务。依据过时即标 stale，不修；再有任务需要就重下。

**来源只有一个**：做任务的 AI。它用任何工具读现实（grep、LSP、解析器、CodeGraph 都只是读现实的手段），把它判断值得记的写进网：调用关系、格式耦合、为什么、谁必须一起改。信息完备不是预先穷举的，是随任务挣来的：第一次改某函数签名，agent 自己读调用点并写下"f.calls⁻¹ = {a, b, c}，依据读数 X"；之后这条在网里，代码动了被标 stale，碰到它的 agent 核一遍再改。网里能区分"没人记过调用者"与"记过，全部新鲜"。

**身份稳定是唯一硬要求**：更新必须是更新不是追加，否则不会收敛。

---

## 5. 锚（anchor 层）

每个实体的现实坐标一根线：探针读现实 → 读数 → fact address（earned hash，覆盖一切影响输出的输入）。读数变了，状态机把变化分类成方面（签名、逻辑、位置、缺失，或价目的 moved_5pct），所有依据该读数的属性、边、结论被标 stale，governs 它们的 Decision 变 due。只标不改。journal 只增，记读数与写入，用于审计与重放；网是 journal 的投影。

保留的现有机制：探针契约（Found / NotFound / Failure 三分）、fact address 与三条不等式、earned derivation、模糊坐标、只增 journal、saw / shown、check 交回人的回路。撤掉的：状态机里累积的轴位与 accept 清零（stale 是投影）、锚为中心的交付、bindings / links 作为并列真源。

---

## 6. 交付与写回

**入口**：locate 用混合检索（名字、关键词、embedding、最近变化）落到实体，给置信度。落到实体之后一切沿边走。

**enter(action, entity)**：按动作签名遍历，输出一张子图：实体、关系、属性、结论，每项带作者与新鲜度标记（fresh / stale{diff} / 无记录）。序列化成紧凑的三元组与属性，几百到一两千 token。完备来自本体（动作签名规定读什么），小来自只匹配实体。诱饵模块 ingest 消费的是另一个 Format 实体，图按实体区分而不是按邻近。交付不等任何模型调用。

**写回**：任务结束，agent 对它读过、建立过、纠正过的东西做四种操作：新增、更新属性、supersede 边或结论、retire。每次写入带 author 与 rests-on。写入只做 schema 检查：类型存在、关系允许于这两种类型之间。没有判官、没有限速、没有 expected head。交付里直接带实体键与边 id，写回是一次调用。写回是完成任务的一部分：交付末尾列出本任务读过但网里没有的东西，做完前写掉。

**动词**：locate · enter · read · write · check。skill 一屏：网是什么三行、五个动词各一行、"读过的东西写回去"、"stale 的先核实"。

---

## 7. 收敛

机制只有一个：**做任务的 agent 顺手更新它读过的节点和边，过时由锚标出。** 每个任务让它涉及的那片信息更清楚，任务叠加即收敛。矛盾由下一个读现实的 agent 解决，重复由身份稳定排除，噪音由 retire 清除。不需要 worker、判官循环、约束校验、失效重抽、本体演化信号。

唯一值得看的数字：stale 标记是否随任务被清掉；同类任务的 enter 交付里"无记录"项是否递减。

---

## 8. 与 KG、CodeGraph、ontology 的关系

- **它是 KG**，按工业界的定义：实体、类型化关系、属性、来源、时效。区别只在建图的方式：不是解析器穷举，也不是流水线批量抽取，而是做任务的强模型按需写入，锚给时效。
- **本体是接口**：agent 通过本体知道世界里有什么、能做什么、该记什么，和 Palantir 的 agent 通过 Ontology 操作企业世界同一个意思。
- **CodeGraph 不是依赖**：它是外部的、无本体、无依据、不受治理的索引。它能表示的信息在这张网里应该看得到，以挣来的形式。agent 可以把它当读现实的工具之一。

---

## 9. 从现状到这里（KISS 顺序）

1. **本体 v1**（人写，一个文件）：coding 域的对象类型、关系类型、动作签名、约束。
2. **存储**：entities · edges · claims · readings · journal 五张表，键由本体定义，网是 journal 的投影。现有 sqlite、探针、fact address、journal 纪律原样复用。
3. **动词**：locate / enter / read / write / check；enter 按动作签名遍历并序列化；write 做 schema 检查；check 报 stale 与 due。
4. **锚接入**：探针读数变化 → 依据它的事实标 stale；Decision 的 due 走现有 check / accept --why 回路。
5. **迁移 memories/**：AI 把 197 篇笔记拆成 Decision / Policy 实体与边，人审；散文进理由字段。
6. **hook**：编辑前 enter，任务结束 write + check。
7. **验收**：channel 与 emergence 的 fixture，测完备性、体积、污染；连续跑同类任务，看 stale 清除率与"无记录"递减。

外部参照：[Palantir Ontology](https://www.palantir.com/docs/foundry/architecture-center/ontology-system) · [OaK 动态本体](https://arxiv.org/html/2608.22974) · [Graphiti](https://neo4j.com/blog/developer/graphiti-knowledge-graph-memory/) · [RANGER 代码 KG](https://arxiv.org/pdf/2509.25257) · [GraphCodeAgent](https://arxiv.org/pdf/2504.10046) · [CodeCompass](https://arxiv.org/pdf/2602.20048)
