# GMR 目标设计

本文是 GMR 目标架构的唯一文档。它取代 2026-09-07 的 rev2、2026-09-08 的 rev3 审查及其附件、2026-09-09 的 rev4，那些文件已删除。本文只写设计是什么，以及落实它要动哪些东西；它不写曾经的版本。

`docs/ARCHITECTURE.md` 描述已发布的实现（0.6.4）。本文描述目标。每一阶段落地后，ARCHITECTURE.md 的对应章节改写为落地后的事实。两者冲突时，ARCHITECTURE.md 说的是现状，本文说的是目标，不存在第三种读法。

本文的每一句话属于三类之一，并在句首标记：

- 【现状】对现有代码、数据或外部工具的陈述。日期 2026-09-09，分支 design/ontology-v1，工作副本 `.anchor/state/`。附录 A 给出每一项的核实命令；命令跑出来不一致，本文错。
- 【决定】owner 于 2026-09-09 的访谈中做出，第 18 节汇总。改动它们走 CLAUDE.md §7。
- 【设计】由决定推出的规格，可以直接实现。

`coding-v1.yaml`、`migration-trial.md`、`migration-full.md` 保留为迁移的输入；它们与本文冲突之处，以本文第 11 节的 v1.1 清单为准。

---

## 1. 定位

【决定】GMR 是一个部署的**接地断言底座**：它持有带来源和 rests_on 的断言，永远不持有散文。它不管会话、不管检索排序、不管按用户或会话分域、不做从对话里抽取记忆。这四样是宿主和读模型的事。

【现状】主流 AI 工具管理记忆的方式一致：会话是宿主的 transcript，长期记忆是一个小文本文件目录，模型用文件工具读写，记忆在会话开始时被注入上下文。Claude Code 载入 CLAUDE.md 和 MEMORY.md 前 200 行；Claude API 的 memory tool 让模型对 `/memories` 发 view、create、str_replace、insert、delete、rename 六个命令，应用把前缀映射到自己的存储；Managed Agents 把 memory store 挂成目录；ChatGPT 在会话开始时注入 saved memories；OpenAI Agents SDK 用 Session 存历史。没有一家把一条记忆绑到可重算的事实上，没有一家在世界变了之后标记它。GMR 住在这一格。

【决定】GMR 不改变用户使用 AI 的逻辑。用户只说话，AI 通过工具动。GMR 因此以工具的形态出现（第 10 节），任务边界由宿主的 hook 接，GMR 不定义任务也不定义会话。

【决定】最小充分信息集和完备性不是设计出来的，是网被连续使用后长出来的动态平衡。GMR 只保证原语的规范和一个标准的读入口；最小性留给读模型。GMR 不声称完备，也不让 AI 声明完备。

---

## 2. 不变量

CLAUDE.md §1 的十三条里十二条原样保留。第 9 条改写为：

> 9. 断言以 rests_on 绑定到锚状态的路径上。rests_on 说"依据什么"；stale 说"依据变了"。何时交回是读模型的事，基座只标记。

其余保留的条目里，与本文直接相关的：第 1 条锚是状态机 δ(state, obs, taken_at, entered_at)；第 5 条版本是 earned hash；第 7 条转移条件只取四个输入；第 8 条终态不可逆；第 10 条规则表首匹配；第 12 条物理分层；第 13 条表达式能构造完整 state。

---

## 3. 一张网，三张图

三张图合成一张可沿边走的网，每条边标明属于哪张图。按谁写、怎么变划分，不按存储划分。

| 图 | 节点 | 边 | 谁写 | 怎么变 | 【现状】底座 |
|---|---|---|---|---|---|
| 事实图 | 实体，带观察属性 | 只有探针能算的结构关系：calls、imports、contains、行情成分 | 只有探针 | 按读数机械重算，永远是可删的本地缓存 | ast-map 以 callee 文本为键产出 call 和 import 候选，无解析器；survey 索引存 posting |
| 锚图 | 传感器 = (实体, 探针, δ) | watches 指向实体；rests_on 来自断言 | 机械 | 状态是读数序列经 δ 的投影 | journal + fold，847 个锚 |
| 记忆图 | 断言：槽位值、边、Claim、Decision、Concept | 领域本体声明的种类 | agent 和人 | 四个写入原语 | `Claim::Said` + `Binding` + `Link`，bindings 表 2,579 行，links 表 388 行 |

【决定】记忆图的底座今天就在。本文对它做的是收紧写入口：粒度、种类、身份、依据、校验。不是另起一张图。

【决定】探针漏掉的结构关系，agent 不能补进事实图，只能写成记忆图里另一种带来源的断言边。

---

## 4. 记忆图的原语

### 4.1 断言

【决定】断言是记忆图唯一的节点粒度。一条断言是一个槽位值或一条边。笔记文件、memory store 里的文件、表里的行，都只是断言的容器，不承担图语义。

【设计】断言记录的字段：

| 字段 | 含义 | 必填 |
|---|---|---|
| kind | `slot` 或 `edge` | 是 |
| subject | 实体 id（4.3）或核心类型节点的 id | 是 |
| name | 槽位名，或关系种类名 | 是 |
| value | 槽位的值；边的对端 id | 是 |
| author | 写入者的声明：`agent <task>` 或 `human <name>` | 是 |
| source | 来源类，见 4.5 | 是 |
| at | 写入时刻 | 是 |
| rests_on | 零到多个依据，见 4.4 | Claim 必填，其余可空 |
| hash | 记录本身的内容 hash，CAS 用 | 由存储计算 |

一条断言的身份是 (subject, kind, name, value 对边而言)。槽位在同一 subject 上只有一个值；边在 (kind, from, to) 上只有一条。

### 4.2 四个写入原语

【决定】写入只有四种，权限只有一条规则。

| 原语 | 语义 | 拒绝条件 |
|---|---|---|
| add | 新增一条断言 | 同身份已存在且未 retire |
| replace | 换掉一条断言的值，携带被换记录的 hash | hash 与当前不符，回复"你没读当前值" |
| retire | 退出交付，进历史，永不删除 | 权限 |
| re-attest | 在 stale 之后确认旧值仍成立：换掉 rests_on 里记录的 hash，不换值 | 权限 |

权限：author 为 agent 的断言，任何 agent 可以 retire 或 replace。source 为 decided 的断言，只有 author 为 human 的写入可以 retire 或 replace；agent 只能对它写 contradicts 边。

【决定】工具不验证谁是人。author 和 source 是声明，写进记录，可审计。一个 agent 在用户完全授权下写 `human`，那是用户的授权，不是工具的事。谁算"人"由领域配置（第 11 节）。

### 4.3 身份

【决定】实体的 id 是 `AnchorKey`，即首次声明时的坐标串，从此按不透明 id 对待，永不重设。label 是 `state.position`，由领域设定。

【现状】`AnchorKey` 就是坐标串（memories/README.md："锚的 key 就是这个字符串"）。`position` 由 coord.rs 从坐标造出，探过一次后能补上 shape。journal、binding_anchors、settings、queue、sighting 五张表以 AnchorKey 为键；links 以记忆的 Ref 为键。

【设计】三种变化，三条路：

| 解析器的判断 | 机制 | 说明 |
|---|---|---|
| 改名或搬家，候选唯一且 shape 相符 | `Revise::Restate` 换 position，理由由解析器机械生成 | key 不动，一张表不动 |
| 不是同一个（rev4 的 replaced：absent 之后 sig 不同） | 旧锚 close，新锚 `supersedes` 旧锚，理由机械生成 | 走现有 `Anchor.supersedes`；交付向后走链，绑定落在活着的世代 |
| 歧义，候选多于一个 | 不动，报候选集 | 解析器不猜 |

【设计】locate 维护一个可删可重建的派生索引：label 到 key，含历史 label。人写的 `about:` 和 agent 写的 `@Type key` 都是 label，写入时经此索引解析为 key；解析不到即 unresolved，拒绝。

代价要知道：key 在改名后写着旧名字。任何把 key 当名字打印的地方改成打印 position。【现状】`read --json` 已经只打地址不打名字。

### 4.4 rests_on 与时效

【决定】一条断言的 rests_on 是若干依据，每个依据是：锚 key，加零到多条 state 路径，每条路径带写入时该路径值的 content hash。不写路径的依据退化为整个读数的 fact address。记录自包含，读时不折 journal。

【设计】stale 判定：对每个依据，取锚当前投影，在记录的每条路径上算 hash，与记录不同即 stale。退化情形比较 fact address。stale 是标记，带理由：

| 理由 | 判定 |
|---|---|
| value | 同一 derivation 版本下，路径上的值变了 |
| instrument | derivation 版本变了，且路径上的值变了；分不开世界动了还是尺子换了 |
| absent | 路径在当前投影里不存在 |

仪器换了但记录的路径上值相同，不 stale。新仪器多出的路径不在记录里，不影响。【现状】这三条就是今天 `read.rs` 的 `differing` 和 Incomparable 的规则，只是从"整个 state 的 diff"换成"记录的路径"。

【决定】时效只有这一种机制。stale 只标不改。rests_on 为空的断言永不 stale，只能被 supersede。时间衰减、使用衰减不进核心。

【决定】stale 的断言交不交付，是读模型的默认值，不是核心规则。核心做两件事：标记，以及让读入口能按 stale 过滤。最简单的读模型交付全部并带标记。

【设计】re-attest 只重写依据里的 hash。它不动锚，不动值。

### 4.5 来源

【设计】source 是核心定义的类别集合，领域在此之上声明自己的形式：

| 类别 | 含义 | 附带 |
|---|---|---|
| observed | 从探针读数得出 | 依据（4.4） |
| decided | 人的判断 | 人的名字 |
| reviewed | agent 写、人在某个发布产物里接受 | 产物标识，coding 里是提交 |
| user-said | 用户说的 | 无 |
| published | 外部世界发布的，如行情或法条 | 发布时刻 |

槽位声明它接受哪些类别。Decision 的 claim 只接受 decided。Claim 的 text 只接受 observed。

【现状】今天的 `Source` 枚举是 Derived、SelfAttested、Adjudicated、Configured、Unknown。

【设计】迁移时的映射：Derived 与 Adjudicated 对应 reviewed，SelfAttested 对应 observed，Configured 与 Unknown 不再产生。

### 4.6 核心本体

【决定】跨领域不变的只有三个类型和五种关系：

- **Concept**：把不相干实体耦合起来的共享物。realizes: 实体 → Concept。两个实体因为共享某物必须一起改，那个物是节点，不是一条带理由的边。
- **Decision**：人的带理由判断。槽位 claim、rationale、review_question、valid_from。decided_by: 实体 → Decision。
- **Claim**：agent 的任务结论。带 task（来源标注，不是检索键），rests_on 必填，依据过时即 stale，从不被静默提升。

关系：rests_on、supersedes、contradicts、realizes、decided_by。

Policy、Incident、Owner 是组织模块；Format、produces、consumes 是数据形状模块。coding 和 ATA 各自导入，个人助理和科研不导入。

【决定】领域本体为空时核心必须能运转：agent 能读现实拿地址，写 Claim rests_on 地址，写 Concept，有人时写 Decision。实体类型和槽位在领域声明之前不存在。agent 遇到本体表达不了的东西，写一条 Claim，内容是"需要一种 X 关系，因为……"，带它当时想连的两个实体；同类提案积累到人能看见时，人改本体出新版本。本体从 Claim 里长，人只做命名。

【设计】Decision 的 review_question 只在被治理实体的锚动了时才该被问，所以 Decision 也应带 rests_on。迁移时从 about 和 watch 合成（第 13 节）；新写的 Decision 由写入者给出，或由 write 从 decided_by 的对端实体当前读数自动填入。

---

## 5. 锚

### 5.1 锚是状态机，保留

【决定】锚是传感器，(实体, 探针, δ) 三元组。一个实体可以有多个锚。δ 保留四个输入，规则表首匹配，终态由锚声明、基座机械执行。

一位 diff 标记做不到、δ 能做到的五件事：往返（越过 5% 又回来）、缺席再现（删了又恢复，hash 等于基线）、驻留（编辑中每分钟都变，该沉降后交回）、仪器变化（状态带 derivation 身份）、终态（退役后不能静默重开）。

### 5.2 journal

【决定】journal 记 open、transition、attempt、revise、close 五种条目。

| 条目 | 携带 | 说明 |
|---|---|---|
| open | 探针、规则表的 ContentHash、终态集、首次 observation 与 state | 规则表本身存进 sealed |
| transition | observation 与 state | state 是投影的检查点，不是独立真源；真源是读数序列。留它是为了 fold 留在 gmr-core，不重放 δ |
| attempt | reason、code、message | 失败路径必须记录，CLAUDE.md §6 |
| revise | change、rationale 的 ContentHash | 不再封 state 快照 |
| close | rationale 的 ContentHash | 同上 |

【现状】still 条目自 2026-08-25 的提交 1acdb24 起只在失败恢复时写（observe.rs：`Some(ref_entry) if s.attempts() > 0`）；无变化的观察只更新 sighting 表。journal 里现存的 46,777 条 still 全部早于 08-25，是历史遗留。

【现状】open 条目平均 9,955 字节，847 条共 8.2 MB，全部是内嵌的规则表；transition 平均 2,489 字节，11,089 条共 27 MB；sealed 里 3,527 份是 seal_context 封的状态快照，共 9.4 MB，581 份是理由，共 133 KB。

【设计】改动三处：`Anchor.transitions` 从 `Transitions` 改为规则表的 `ContentHash`，规则表存入 sealed，847 份变成约 4 份；`seal_context` 停用，revise 和 close 只封理由；`Retain` 与 still 的现状不变。fold 留在 gmr-core，gmr-core 继续不依赖 gmr-expr。

【设计】读数缓存：sighting 表加 fact_address 列，成为 (anchor, address, hash, at)。

### 5.3 coding pack 的富形状

【现状】shapes.rs 在 console/cli，1,261 行，四个形状 contract、roster、fingerprint、value，共 15 个 dim，全部是 `baseline != obs` 的一位比较。taken_at 和 entered_at 在 shapes.rs 里出现 0 次。gmr-expr 完整支持这两个输入和 `30d` 这类时长字面量；runtime 的 translate.rs 已把 entered_at 传入 δ。

【设计】形状挪进 packs/coding，contract 形状保留现有八个轴 missing、name、file、kind、sig、surface、logic、place，在此之上加：

| 状态 | 进入条件 | 离开条件 | 用途 |
|---|---|---|---|
| flux | 任一轴在 taken_at 相对 entered_at 的驻留窗口内变过 | 驻留窗口内无变化 | 读模型可以选择在 flux 期间不交回 |
| settled | flux 离开 | 任一轴变 | 交回的默认时机 |
| absent | obs.found == false | obs.found == true | 依据它的断言全部 stale |
| returned | absent 离开且读数的 sig 与最后一次 present 的 sig 相同 | 下一次 transition | 报"回来了"，不是新实体 |
| replaced | absent 离开且 sig 不同 | 下一次 transition | 解析器决定是否同一 id（4.3） |

state 另带 moved_count 和 last_moved_at。驻留窗口是 pack 的参数，基座不知道它。roster、fingerprint、value 形状同理加 flux、settled、absent、returned。

三条实现约束，来自 δ 的求值方式：flux 到 settled 靠时间流逝触发，但 δ 只在观察时求值，窗口内无人观察则下次观察一步跳到 settled，"编辑期间不交回"只在 hook 每次编辑都观察时成立；returned 要比对最后一次 present 的 sig，absent 状态必须把它带进 state；flux 期间每次变化都要产生不同的 state 对象，否则 entered_at 不会重置，所以 last_moved_at 写 taken_at。

### 5.4 与现状的差距

| 差距 | 【现状】证据 | 处置 |
|---|---|---|
| 规则表随每次 read 交付 | read --json 一个坐标 18,564 字节，其中 anchor.transitions 9,265 字节（2026-09-08 在 0.6.3 上测得） | 5.2 |
| 锚 = 坐标 = 身份 | AnchorKey 是坐标串，position 不带 shape | 4.3 |
| 绑定在笔记级，订阅靠手写 watch | 196 篇笔记中 175 篇有 watch，145 篇一模一样是 `[sig, logic]` | 4.1、4.4 |
| 只有退化形状 | 15 个 dim 全是一位比较，时间输入未用 | 5.3 |
| 形状放错了层 | shapes.rs 在 console/cli | 5.3 |
| 状态快照被封存 | sealed 9.4 MB 是快照 | 5.2 |
| still 堆积 | 已于 08-25 关闭 | 无 |

---

## 6. 存储

### 6.1 契约

【决定】核心定两样：断言的自包含记录格式（第 9 节的行语法即字节格式），以及存储契约。家由部署定：coding 放仓库文件，ATA 放它 submit 事务里的表，个人助理放本地库或托管服务。共享、审阅、备份是那个家自己的机制。

【设计】存储契约由 gmr-net 声明为三个 trait，任何后端实现：

| trait | 方法 | 【现状】对应 |
|---|---|---|
| Assertions | add、replace(expected_hash)、retire、reattest、by_subject、by_author、all | bindings 表加两列：rests_on 的路径与 hash |
| Edges | out(subject, kind?)、in(object, kind?) | links 表，kind 由本体约束 |
| Readings | put(anchor, address, hash, at)、get(anchor) | sighting 表加 address 列 |

gmr-store 保留 Journal、Chained、Sealer、Queue、Settings、Sightings、Usage、Ledger，其 sqlite 后端实现 gmr-net 的三个 trait。`BindingStore` 和 `LinkStore` 由 Assertions 和 Edges 取代，方法一一对应：bind 是 add，revoke 是 retire，bindings_on 是 by_subject，links_of 是 out，links_to 是 in。

### 6.2 校验器住在 write 里

【决定】校验器住在 gmr-net 的 write，无 IO。所有门都是它的调用者：MCP 工具、memory tool 后端、CLI、SDK、宿主 hook、coding pack 的文件索引、ATA 的 submit 事务。没有一扇门有自己的规则。

家是人可直接改的介质时，那个部署负责把人改过的东西重新过一遍 write。coding 里这叫 index（6.3）。

### 6.3 coding 的家

【决定】沿用 memories/ 的主题文件做容器，正文换成行语法，frontmatter 消失。

【设计】容器格式：一个 `.md` 文件是一个行语法文档，可以有一行 `#` 标题作为人读的题名，解析器跳过它。文件里每一块以 `@Type key` 开头，块内是槽位行、边行，文档末尾是来源脚注。一个文件可以放多个块。about 和 watch 不再存在：它们变成每个来源脚注里的 rests_on。

【设计】index 取代 sync，单向：从文件重建本地索引，对每一行跑同一个校验器。索引条目按文件 hash 门控，和 extract-cache.json 一个形状；人改了文件，旧条目在任何读之前失效。文件里校验不过的行进索引标 unresolved，doctor 和 check 报出，退出码 1，永不静默消失。本体升版后旧行不再合法也走这条路。

write 落到 coding 的家时，先校验，再把行写进容器文件，再更新索引。【现状】`gmr anchor -m` 今天就是先写笔记文件再跑 sync，顺序相同。

CAS 只保护本地写路径。两个分支各自 replace 同一槽位，git 合并时是文件冲突，不是被拒绝。这是"共享和审阅是那个家自己的机制"的具体含义。

---

## 7. 校验器

【设计】write 接一个文档，多块，原子。校验顺序如下，任何一条不过则什么都不写，回复指名行号和代码：

| 代码 | 检查 |
|---|---|
| E01 | 类型在本体里 |
| E02 | 键形合法 |
| E03 | 事实键的实体现在路由到解析器并命中；未命中即 unresolved |
| E04 | 槽位已声明 |
| E05 | 创建时 required 槽位齐全 |
| E06 | 来源对该槽位合法；Decision.claim 只接受 decided |
| E07 | observed 引用的地址是本任务里 read 或 walk 发过的 |
| E08 | 一句话：单终止符，不超过 240 字符 |
| E09 | 关系种类存在 |
| E10 | 关系两端类型匹配；must_change_with 必带 reason；supersedes 指向同类已有目标 |
| E11 | replace 携带的 hash 与当前记录相符 |
| E12 | 权限（4.2） |

之后是噪音检查。N06 只警告，其余拒绝：

| 代码 | 检查 | 抓的是 |
|---|---|---|
| N01 | 历史词：used to、once、previously、no longer、briefly、originally、has not changed | 历史 |
| N02 | ISO 日期；`\d+ ?(ms\|KB\|MB\|%\|of \d+)`；Config.controls 与 Field.unit 豁免 | 某天的测量 |
| N03 | `[[…]]`、`](`、"see "、"as X says" | 写成散文的引用，应是边 |
| N04 | 与实体源码重合不少于 40 字符，或反引号段多于 2 个 | 复述代码 |
| N05 | claim 里的 we、I、probably、seems | 日记 |
| N06 | 反引号标识符必须在实体读数里或能解析为键 | 从写下起就错的话；警告 |
| N07 | claim 里的 because、so that、since | 理由漏进 claim |
| N08 | 与已有槽位 stem-Jaccard 不低于 0.8 且无 supersedes | 复述 |

回复不超过 200 字节。成功列出落地的节点与边数；退出码 0 成功，1 有警告已落地，2 拒绝未写。

抓不到的：来源合法的一句话假合同。那是 entailment，是 agent 的事，不是工具的。

---

## 8. 读入口

【决定】一个标准协议：取一个节点；按种类、方向、来源类、stale 状态、实体类型取边，限深度和预算；取锚的当前状态。外加最简单的沿边查询。读模型不是数据，是用户按这个协议自己开发的检索算法；核心不提供路径语言，不存读模型。

| 动词 | 做什么 | 输出上限 |
|---|---|---|
| locate | 词、label 或位置到实体；用 4.3 的派生索引 | 5 行 |
| read | 一个节点，或 `ontology.Type` 列出槽位与关系 | 1 KB |
| walk | 从一个节点按过滤条件取边，一跳或限深 | 4 KB，信封不超过 20%，截断显式计数 |
| write | 四个原语 | 200 字节 |
| check | 哪些断言 stale，哪些锚 transition；安静时零输出 | 每项约 100 字节 |

walk 的过滤参数：kind、direction、source class、stale（include、exclude、only）、entity type、depth、budget。stale 的默认值由调用方的读模型定，基座的默认是 include 并标记。

【决定】以下不是核心的事：按 task 取回一个任务的 Claim（task 只是来源标注；读模型可以按 author 过滤）；按会话分域；语义检索。

---

## 9. 协议

### 9.1 行语法，agent 读写共用

```
@Type key [=addr] ~s          节点行。key 是 label；=addr 是该实体最近读数地址的前 8 位 hex
slot: text ~s                 槽位值。读输出里行首 ! 表示其依据在锚上变了
>rel Type:key ~s              出边；<rel 入边。读输出里缩进行是对端节点的槽位
~s who how <basis>            来源脚注，每种来源只写一次，放页脚
```

`<basis>` 按来源类：

```
observed  key@addr[path,path]   addr 是本任务里 read 或 walk 发过的地址；方括号列依赖的 state 路径，
                                省略即整个读数。存储时 runtime 从该地址的读数取出各路径的 hash 记录下来
decided   <name> <date>
reviewed  <commit>              coding 里 index 也可从 git blame 解析
user-said
published <time>
```

读输出的脚注在 stale 时追加 `!stale <path,…> @<seq>`，seq 是使依据变化的 transition。

一个 walk 输出的例子（改写自 2026-09-08 审查的示例，脚注按本节格式；实体与文字取自本仓库）：

```
@Function crates/gmr-runtime/src/memory.rs#carry_linked =fcfcce7f ~a
purpose: carries records linked from an already-delivered memory into the answer, one hop, when asked. ~a
!contract: gated by Instructions.carry; narrows a slice of the caller's total Budget per record and mints no total of its own. ~a
>reads Field:crates/gmr-runtime/src/read.rs#Instructions.carry ~a
>decided_by Decision:carry-one-hop ~b
 claim: relevance beyond one hop is the domain's judgment, not the substrate's.
 ask: does the new code recurse past one hop, or walk without carry being asked?
<calls Function:crates/gmr-runtime/src/read.rs#ground ~p
~a agent task-0912 observed crates/gmr-runtime/src/memory.rs#carry_linked@fcfcce7f[now.sig,now.body] !stale now.body @62437
~b human zongming decided 2026-08-22
~p probe callers now
```

### 9.2 结构对象路径

程序路径用结构对象，不过文本：断言记录的字段（4.1）直接序列化。【现状】console/core 的 Opening、opened、said、answered、asking 是今天的结构化门面，node 和 python 两扇门走它。

---

## 10. 门

【决定】一个核心，三类门，形态照抄 AI 今天用记忆的方式。

### 10.1 MCP server

agent 的通用门。五个动词各一个工具：`locate`、`read`、`walk`、`write`、`check`。输入是文本参数，输出是行语法文本。工具描述就是入口卡，不另发 skill 散文。【现状】仓库里没有 MCP server；Claude Code 的 skill 是 28,258 字节的散文告诉 agent 去跑 CLI，只有 Claude Code 一个宿主能用。

### 10.2 memory tool 后端

Claude API 应用的门。一个 `/memories` 处理器，六个命令映射到原语：

| 命令 | 映射 |
|---|---|
| view /memories | 列出类型 |
| view /memories/Type | 列出该类型的 key，有上限 |
| view /memories/Type/key | read 该节点，行语法带行号 |
| view /memories/Type/key/walk | walk 一跳，默认过滤 |
| create | write：file_text 是行语法文档 |
| str_replace | replace：old_str 必须逐字匹配一整行且唯一，那一行的记录 hash 就是 CAS |
| insert | write add |
| delete /memories/Type/key | retire 该节点上 author 有权 retire 的全部断言 |
| rename | 拒绝：label 是 position，身份是 key |

【现状】memory tool 的 str_replace 本身要求 old_str 逐字出现且唯一，否则拒绝。这就是 CAS，主流接口自带。系统提示里"先 view 记忆目录"那句替 GMR 说了任务开始时的 walk。

### 10.3 CLI 与 SDK

程序和操作员的门，结构对象路径。init、probes、export、import、publish、revise 族藏在这里。【现状】CLI 有 16 个显示动词和 18 个隐藏动词；其中 said、bind、attest、reaffirm、cobound、link、condense、`anchor -m` 并入 write；read、since、links、list、memories、status、ground、standing、sample、health 并入 read 和 walk；check、observe、pass、doctor、sync 并入 check 和 index；adopt、atlas 动词、`accept --baseline` 删除；`accept --criteria` 就是 revise。其余隐藏动词在 Phase 2 逐个归类，本文不预先断言。

### 10.4 宿主接法

| 宿主 | 任务开始 | 任务结束 |
|---|---|---|
| Claude Code | UserPromptSubmit hook 跑 locate 与 walk，stdout 注入上下文 | Stop hook 跑 check |
| Claude API memory tool | 系统提示自带的"先 view" | 应用自己的循环 |
| Managed Agents | 挂载描述注入 | 同上 |
| MCP 客户端 | 工具描述 | 无 |

---

## 11. 领域

【决定】领域本体提供：实体类型与键形；解析器；探针与地址函数，什么变化算变化写在这里；形状即 δ；来源形式；关系词汇；谁算人。种子读模型是可选的外部程序。

| | coding | 交易论证 | 法规与合同 | 个人助理 |
|---|---|---|---|---|
| 实体 | Module、Function、Type、Test | Instrument、Thesis、Evidence | Statute、Clause、Obligation | Person、Food、Place |
| 探针读什么 | AST、文件 hash | 行情 bar | 法条文本版本 | 冰箱清单、日历 |
| 什么算变了 | 签名、正文、位置分轴 | tolerance policy 分桶后的价格加 interval | 版本号 | 清单差异 |
| 来源形式 | observed、reviewed:commit、decided | observed、published、decided | published、decided | user-said |
| 谁算人 | owner | owner | 律师 | 用户本人 |

### 11.1 coding v1.1

【设计】coding-v1.yaml 是输入，v1.1 在它之上改十四处：

1. Function 的键从 `path#name(shape)` 改为 4.3 的 id，shape 是观察属性。
2. 每种关系标 `source: observed` 或 `asserted`；calls、imports、contains、implements、reads、exposes 从 relations 移到 readings，由探针物化，write 拒绝 agent 写它们。
3. 加 Concept、realizes、`Policy.about → Concept`；must_change_with 保留为兜底且必带 reason，同一 reason 第二次出现时 write 提示命名为 Concept。
4. Claim 的 standing 由 stale 推得，about 必填改为 rests_on 必填。
5. 每实体每关系的 `complete` 声明，作为负信息交付。
6. Decision 和 Policy 加 standing，值有 live 与 retired。
7. 加 Dependency 实体，键为 crate 或服务名。
8. 加 File.role 槽位。
9. Test 可以是 governed_by 与 decided_by 的起点。
10. 加 enforced_by 关系，Policy → Function、Config、Test。
11. 加 Constant 实体，键 `path#NAME`。
12. 加 requests 关系，Function → Endpoint。
13. Config 的键允许 `env#NAME`。
14. 关于 Claim 如何书写的规则是本体文件的公理，不是 Policy 实例。

其中 1 到 5 来自 2026-09-08 的审查，6 到 14 来自 migration-full.md 的 v1.1 清单。

### 11.2 coding 的探针

【现状】四个内建提取器 ast-map、addr-map、name-map、prose-map，加一个脚本探针 test-roster。解析器按 file、kind、name、shape 逐项收窄，任何一项失败不终止，最后按匹配项数取最优候选，报 candidates 数。

【设计】name-map 删除，其职责由 4.3 的 locate 索引承担。survey 的 posting 表不再把文件正文放进 callee 列；索引缩成按文件 hash 门控的 locate 索引。survey 的 walk、corpus、narrow、matching、recipe 原样保留，extract 依赖它们。

---

## 12. 生态

【决定】外部记忆库与 GMR 只有三种关系，没有一种是"GMR 读它们的内容当记忆"。

| 关系 | 机制 | 状态 |
|---|---|---|
| 做家 | 实现 6.1 的三个 trait。mem0 能存记录，但其关系不可遍历，所以边和 rests_on 索引由 GMR 本地留可删缓存 | 需要时写 battery |
| 做 rests_on 目标 | AI 在 GMR 之外写的东西，auto memory 文件、mem0 记录，对 GMR 是现实：一个探针读它的内容 hash，Claim 可以依据它 | file 传输 226 行已在；mem0 用 script 探针 |
| 做读模型 | 外部检索建在 GMR 断言之上做排序 | 用户自己接 |

【现状】今天的 gmr-content 与提供者是第四种关系：GMR 读外部记忆的正文，只记绑定。本仓库里这条路零使用（绑定全部是 git 笔记和 said，providers.toml 不存在），且它无法校验、无法 CAS、无法为没发过地址的东西算 rests_on。这条路删除。

【现状】mem0 2026 的形状：add、search、update、delete；LLM 抽取记忆；检索是语义、BM25、实体匹配融合；按 user、session、agent 分域；关系字段已去掉。这些里 GMR 有意不做的三样是抽取、检索排序、分域。

---

## 13. 迁移

【决定】迁移出的判断以 `reviewed:<合并提交>` 为来源；owner 审阅迁移 PR 并合并即签字。

【设计】步骤：

1. 校验器和行语法渲染器先存在（第 15 节 Phase 0）。
2. 按 migration-full.md 的逐篇拆分生成容器文件：Decision 与 Policy 节点带 claim、rationale、ask；从 about 实体出发的 decided_by 或 governed_by 边；槽位值；边。
3. 每条 Decision 与 Policy 的 rests_on 从 about 与 watch 机械合成：about 的每个坐标给一个锚 key；watch 的轴映射为 state 路径，sig 到 now.sig，logic 到 now.body，name 到 now.name，file 到 now.file，kind 到 now.form，surface 到 now.surface，place 到 now.after，roster 的 grew 与 shrank 到 count、roll 到 roll，fingerprint 的 drift 到 fingerprint；hash 取迁移当时的值。
4. 关于被删机制的 52 篇，判断照迁，standing 标 retired。
5. 每一行过 write。过不去的列表交 owner。
6. owner 审 PR，划掉的条目降为 agent Claim 或丢弃，合并。
7. 需要 v1.1 才有槽位的 13 篇进第二批。

【现状】memories/ 的 50 次提交全部由 owner 提交，所以每篇笔记在 git 里已经是 owner 审过的东西。migration-full.md 拆出 714 条 Decision 与 Policy，其中 Policy 175 条、带 review question 305 条；499 个槽位值，1,049 条边；131 篇按原样可入槽，52 篇关于被删机制，13 篇需要 v1.1；单模型一遍、未经复核，714 是下限。

【现状】今天 bindings 里带 depends 的 10 条中 9 条引用了 `state.v.*` 粘滞位。迁移后 `v.*` 消失，这 9 条的语义变为对应的 now 路径。

---

## 14. 证伪实验，先于 runtime 代码

【决定】子系统换成 gmr-core。10 个任务就是 5.2 的真实工作：规则表内容寻址、停封状态快照、sighting 表加地址列、attempt 语义保持、相关测试。证伪实验同时是 Phase 1 的交付。

【现状】存活子系统里笔记最密的是 gmr-core，42 篇；其次 gmr-store 40 篇，packs/coding/extract 29 篇，gmr-expr 17 篇。rev4 选的 read.rs 有 39 篇，但它是 Warrant、Holding、Shown 所在地，即将拆除。

【设计】

- 校验器脚本先行，跑在行语法上，实现第 7 节的全部检查。它之后是 gmr-net write 的规格。
- 子图臂的网初始只有从 gmr-core 的 42 篇笔记迁出的 Decision，按第 13 节的机制。对照臂用今天的 `read --json` 加 SKILL.md。每个任务一次 fresh `claude -p`，两臂同模型同顺序。
- 臂的隔离和记录复用 tools/channel：`--allowedTools` 锁工具，stream-json 记录每次工具调用。"交付集之外读的字节"等于流里 Read、Grep、Bash 读取的结果字节减去 walk 输出。
- 每任务测四个数：首次编辑前在交付集之外读的字节；测试通过率；写入里无提示即合法的比例；任务结束后的 stale 断言数。第四个只记不判。
- 预注册判据：后 5 个任务的外部读取字节均值低于前 5 个的 70%，且无提示合法写入不低于一半。任一不满足即证伪。

---

## 15. 实施顺序与验收

### 零设计风险，可以现在做

1. `read --json` 停止输出规则表。只改 CLI。
2. 在 pack 里编写 flux、settled、absent、returned、replaced 形状，对现有求值器跑测试。不动基座。
3. shapes.rs 挪进 packs/coding，连同它依赖的 rules、contract、probes::Obs 类型。

### Phase 0：证伪

产出：校验器脚本、行语法渲染器、第 14 节的实验记录。验收：预注册判据。

### Phase 1：gmr-net、write、index、迁移

产出：crates/gmr-net，含记录格式、第 7 节校验器、四个原语、三个存储 trait、读入口算法；gmr-store 的 sqlite 后端实现三个 trait，bindings 表加两列，sighting 表加一列；index 取代 sync；第 13 节迁移；CLAUDE.md 与 gate.py 按第 17 节改。验收：write 拒绝无槽位文本；196 篇迁移完成并报出丢弃率；gate 通过。

### Phase 2：walk、check、门

产出：walk、check、locate 建在新存储上；stale 比较器取代 Warrant、Holding、Shown、Depends；MCP server；memory tool 后端；SKILL.md 缩成入口卡；5.2 的 journal 改动。验收：walk p90 不超过 4 KB；提交后的 check 是 O(触及文件数)；MCP 门跑通 tools/msis 的四个场景。

### Phase 3：身份与删除

产出：解析器确认改名时的 Restate；locate 的 label 索引；第 16 节的删除。验收：gate 与全部测试通过；survey 索引与 memory.db 的体积按第 16 节下降。

---

## 16. 保留、修正、删除

行数为 2026-09-09 计数，只含 src。

| 项 | 行数 | 处置 |
|---|---|---|
| gmr-core journal 与 fold | 909 | 保留；5.2 的三处改动 |
| gmr-expr | 1,835 | 保留 |
| gmr-budget | 220 | 保留 |
| gmr-probe | 130 | 保留 |
| gmr-store | 3,564 | 保留；BindingStore、LinkStore 由 gmr-net 的 Assertions、Edges 取代，表不动 |
| gmr-runtime read.rs | 1,375 | 修正：Warrant、Holding、Shown、Depends、Footing 删壳，differing 与 Incomparable 规则搬进 stale 比较器 |
| gmr-runtime 其余 | 3,687 | 修正：bind、condense、reaffirm 成为四原语的实现；edges、link 成为 walk 的实现 |
| gmr-content | 360 | 删除整个 crate |
| batteries/provider | 1,708 | 删除整个 battery |
| batteries/transport http、sql | 940 | 删除 |
| batteries/transport 其余（inproc、script、file、shell、template 等） | 2,220 | 保留 |
| batteries/survey index、sqlite | 865 | 修正：缩成 locate 索引，不存文件正文 |
| batteries/survey 其余 | 1,750 | 保留 |
| batteries/atlas | 398 | 保留：改成吃 walk 输出的读模型 |
| packs/coding/extract | 2,414 | 保留；name-map 删除 |
| console/cli shapes.rs | 1,261 | 挪进 packs/coding |
| console/cli sync.rs、memories.rs、notes.rs | 约 2,750 | 修正：变成 index 与容器解析 |
| console/cli adopt.rs | 281 | 删除 |
| console/cli atlas.rs | 447 | 修正：渲染 walk |
| console/cli 其余动词 | 见 10.3 | 并入五个动词或藏为操作员动词 |
| console/core、node、python | 906 | 保留；加 MCP 门 |
| .claude/skills/gmr/SKILL.md | 28,258 字节 | 缩成入口卡 |
| .anchor/state/survey-index.sqlite | 353 MB | posting 表 362,274 行不再存文件正文 |
| .anchor/state/memory.db | 105 MB | open 条目 8.2 MB 与 sealed 快照 9.4 MB 消失；still 遗留 16 MB 随重建消失 |

---

## 17. 边界与门禁

【设计】CLAUDE.md：

- §1 第 9 条改为第 2 节的文字。
- §2 memories/ 的描述改为"记忆图在 coding 部署里的容器"，about 与 watch 的说明删除。
- §5 加一条：**gmr-net**：断言记录格式、校验器、四个写入原语、读入口算法；声明 `Assertions`、`Edges`、`Readings` 三个存储契约。无 IO，依赖 gmr-core，不依赖 gmr-store 与 gmr-runtime。
- §5 删 gmr-content 一条；gmr-store 一条的 trait 名单去掉 BindingStore 与 LinkStore；gmr-runtime 一条的依赖列表以 net 替 content。

【设计】tools/gate.py：`TRAIT_ROSTERS` 以 gmr-net 替 gmr-content；`NO_CONCRETE_IMPL` 加 gmr-net，去 gmr-content；`CLEAN_ZONES` 加 crates/gmr-net，去 crates/gmr-content、batteries/provider；`architecture.toml` 的排除表同步。

【设计】文字改写：SKILL.md 与 ARCHITECTURE.md §8 里"GMR 不存记忆内容"改为"GMR 只存断言，不存散文"；memories/README.md 改为 6.3 的容器格式。

---

## 18. owner 决定记录，2026-09-09

| # | 决定 |
|---|---|
| 1 | 记忆图的底座今天就在；本文是收紧写入口，不是重做 |
| 2 | 领域本体为空时核心靠 Claim 加地址运转；本体从 Claim 里长 |
| 3 | 会话、恢复、任务状态是宿主的；task 是来源标注；stale 只标，交付是读模型的默认值 |
| 4 | 断言是唯一节点；文件是容器 |
| 5 | key 就是 id，label 是 position；改名走 Restate，replaced 走 supersede |
| 6 | rests_on 是锚 key 加路径加值 hash，自包含；不写路径退化为地址 |
| 7 | 校验器住在 gmr-net 的 write，所有门都是调用者 |
| 8 | 门是 MCP、memory tool 后端、CLI 与 SDK |
| 9 | 规则表内容寻址，快照停，transition 保留 state 作检查点，attempt 保留 |
| 10 | links 表、bindings 表、atlas 保留并约束；gmr-content 整个删除 |
| 11 | GMR 是接地断言底座；外部记忆库是家、rests_on 目标或读模型 |
| 12 | 迁移来源 reviewed 合并提交；校验器做第二遍 |
| 13 | coding 的家沿用 memories/ 主题文件 |
| 14 | 证伪实验换 gmr-core，校验器先行，判据预注册，复用 channel |

其中按 CLAUDE.md §7 属于 owner 的四项，在此明记：attempt 留在 journal；删除 gmr-content 与 batteries/provider；新增 gmr-net；fold 留在 gmr-core，不重放 δ。

---

## 附录 A：核实命令

所有命令在仓库根目录执行，日期 2026-09-09。

| 陈述 | 命令 |
|---|---|
| journal 各类条目数 | `sqlite3 .anchor/state/memory.db "SELECT substr(body,1,12), count(*), sum(length(body))/1024 FROM journal GROUP BY 1"` |
| still 最后写入时间 | `sqlite3 .anchor/state/memory.db "SELECT max(json_extract(body,'$.at')) FROM journal WHERE body LIKE '{\"entry\":\"still\"%'"` |
| still 只在失败恢复时写 | `grep -n "attempts() > 0" crates/gmr-runtime/src/observe.rs`；`git log -1 --format='%h %ad' --date=short 1acdb24` |
| sealed 快照与理由 | `sqlite3 .anchor/state/memory.db "SELECT body LIKE '{\"at_entry\"%', count(*), sum(length(body))/1024 FROM sealed GROUP BY 1"` |
| seal_context 封什么 | `cat crates/gmr-runtime/src/seal_context.rs` |
| gmr-core 不依赖 gmr-expr | `sed -n '/\[dependencies\]/,/^$/p' crates/gmr-core/Cargo.toml` |
| bindings、links 计数 | `sqlite3 .anchor/state/memory.db "SELECT (SELECT count(*) FROM bindings),(SELECT count(*) FROM links),(SELECT count(*) FROM link_revocations)"` |
| 带 depends 的绑定 | `sqlite3 .anchor/state/memory.db "SELECT count(*) FROM bindings WHERE body LIKE '%\"depends\"%'"` |
| watch 分布 | `cat memories/*.md \| grep '^watch:' \| sort \| uniq -c` |
| 各子系统笔记数 | `cat memories/*.md \| awk '/^about:/{f=1; if($0 ~ /about: [^ ]/){print $2}; next} /^[a-z]+:/{f=0} f&&/^  - /{print $2}' \| sed 's/"//g; s/#.*//' \| awk -F/ '{print $1"/"$2"/"$3}' \| sort \| uniq -c \| sort -rn` |
| shapes.rs 无时间输入 | `grep -c 'taken_at\|entered_at' console/cli/src/shapes.rs` |
| gmr-expr 支持时间输入 | `grep -n "taken_at\|entered_at" crates/gmr-expr/src/parse.rs` |
| 各 crate 行数 | `find <dir>/src -name '*.rs' \| xargs cat \| wc -l` |
| survey 索引表 | `sqlite3 .anchor/state/survey-index.sqlite ".schema posting"`；行数 `SELECT count(*) FROM posting` |
| CLI 动词与隐藏标记 | `grep -n "hide = true" -A1 console/cli/src/cli.rs` |
| console/core 门面 | `grep -n "^pub" console/core/src/lib.rs` |
| memories/ 提交作者 | `git log --format=%an -- memories/ \| sort \| uniq -c` |
| SKILL.md 大小 | `wc -c .claude/skills/gmr/SKILL.md` |
| Anchor.supersedes 与 heir_of | `grep -n "supersedes\|heir_of" crates/gmr-core/src/anchor.rs crates/gmr-runtime/src/bind.rs` |
| read --json 18,564 字节 | 2026-09-08 在 0.6.3 上测得，命令 `gmr read crates/gmr-runtime/src/memory.rs#carry_linked --json \| wc -c`；本文未在 0.6.4 上重测 |
| memory tool 六命令 | https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool |
| Claude Code hook 事件 | https://code.claude.com/docs/en/hooks |
| Claude Code 记忆机制 | https://code.claude.com/docs/en/memory |
| mem0 2026 形状 | https://docs.mem0.ai/migration/oss-v2-to-v3 |
| MCP 客户端支持 | https://contextbolt.com/blog/ai-tools-mcp-support/ |
