# GMR 目标架构 rev4（2026-09-09）

本文是当前设计，取代 rev2（2026-09-07）。它只写设计是什么；论证在 `review-2026-09-08.md` 及其附件里。唯一的例外是第 6 节：锚为什么是状态机、现在的实现错在哪，这一段写进来，因为它决定了后面每个 agent 怎么对待 δ。

---

## 1. 目标

GMR 是基础设施，设计的只是原语。原语让 fact、anchor、memory 三张图都结构化、原子化，让每个做任务的 agent 都能在网上连边和断边。最小信息集和完备性都不是设计出来的：它们不能被证明，任何声称设计出来的都是伪命题或只在窄场景成立。它们是网被连续使用后长出来的动态平衡：有关联的节点被连起来，无关的被取消，同类任务的后来者更容易找到有效信息，任务变了网跟着变。

GMR 因此不声称什么是完备、什么是最小，也不让 AI 声明完备，让它声明就是逼它撒谎。GMR 保证的是原语的规范和一个标准的读入口；最小性留给开放的读模型空间，每个用户自己开发检索算法，或者只做最简单的沿边查询。

---

## 2. 一张网，三张图

三张图合成一张可沿边走的网，每条边标明属于哪张图。三张图按谁写、怎么变划分，不按存储划分。

| 图 | 节点 | 边 | 谁写 | 怎么变 |
|---|---|---|---|---|
| 事实图 | 实体，带观察属性 | 只有探针能算的结构关系：calls、imports、contains、行情的成分关系 | 只有探针，agent 一律不能 | 按读数机械重算，永远是可删的本地缓存 |
| 锚图 | 传感器 = (实体, 探针, δ) | watches 指向实体；rests_on 来自记忆断言 | 机械 | 状态是读数序列经 δ 的投影 |
| 记忆图 | 槽位值、Decision、Claim、Concept | 领域本体声明的种类 | agent 和人 | 四个写入原语 |

探针漏掉的结构关系，agent 不能补进事实图；它只能写成记忆图里另一种带来源的断言边。今天的系统把三样混在一起：links 表全是 cites，事实边靠 callee 文本，笔记正文夹引用。

---

## 3. 原语，跨领域不变

**断言单元。** 一个槽位值，或一条边。每条带 author、source、at，以及零到多个 rests_on。rests_on 指向一个锚的状态里的若干路径。

**四个写入原语。** add 新增；replace 换值，携带所换值的 hash，不匹配即拒绝，所以更新是更新不是追加；retire 退出交付进历史，没有 delete；re-attest 在 stale 后确认旧值仍然成立，只换依据不换值。权限：agent 写的任何 agent 可 retire；人 decided 的只有人能 retire，agent 只能对它写 contradicts。谁算“人”由领域配置。

**身份。** 实体首次被观察时铸造一个不透明 id，永不变。路径、名字、签名形状都是观察属性，wire 上显示的是 label。领域提供解析器：给 label 或位置答 id，读数变了时判断“还是同一个”还是“没了”，歧义时答候选集，不猜。

**时效。** 只有一种机制：断言 rests_on 的锚状态在它声明的路径上变了，即 stale，只标不改。锚状态由领域的 δ 决定，见第 6 节；平凡形状下它退化成读数 hash 比较。rests_on 为空的断言永不 stale，只能被 supersede。时间衰减和使用衰减不进核心：断言带 at，使用有记录，读模型自己算。

**读入口。** 一个标准协议：取一个节点；按种类、方向、来源类、stale 状态、实体类型取边，限深度和预算；取锚的当前状态。外加最简单的沿边查询。读模型不是数据，是用户按这个协议自己开发的检索算法；核心不提供路径语言，不存读模型。

**存储。** 核心定两样：断言的自包含记录格式，任何后端存同一种字节，换后端不换语义；存储契约，必须实现四个写入原语、按实体读、按边双向读、读数缓存。事实图永远是可删的本地缓存。记忆图住哪由部署定：coding 放仓库文件，因为它的审阅和分发通道就是 git；ATA 放它 submit 事务里的表；个人助理放本地库或托管服务。共享、审阅、备份都是那个家自己的机制。

**协议。** agent 路径用读写共用的行语法，程序路径用结构对象。规则不写成散文：write 的拒绝指名行号和代码，read ontology.Type 列出槽位，skill 只剩一张入口卡。

---

## 4. 核心本体，极薄

跨八个领域都出现的只有三个类型和五种关系：

- **Concept**：把不相干实体耦合起来的共享物。realizes: 实体 → Concept。两个实体因为共享某物而必须一起改，那个物必须是节点，不是一条带理由的边。
- **Decision**：人的带理由判断。槽位 claim、rationale、review question、valid_from。decided_by: 实体 → Decision。
- **Claim**：agent 的任务结论。带 task，rests_on 必填，依据过时即 stale，从不被静默提升。

关系：rests_on、supersedes、contradicts、realizes、decided_by。

Policy、Incident、Owner 是组织模块；Format、produces、consumes 是数据形状模块。coding 和 ATA 各自导入，个人助理和科研不导入。

---

## 5. 领域

领域本体提供：实体类型与键形；解析器；探针与地址函数，什么变化算变化写在这里；形状，即 δ；来源形式；关系词汇；谁算人。种子读模型是可选的外部程序。

| | coding | 交易论证 | 法规与合同 | 个人助理 |
|---|---|---|---|---|
| 实体 | Module、Function、Type、Test | Instrument、Thesis、Evidence | Statute、Clause、Obligation | Person、Food、Place |
| 探针读什么 | AST、文件 hash | 行情 bar | 法条文本版本 | 冰箱清单、日历 |
| 什么算变了 | 签名、正文、位置分轴 | tolerance policy 分桶后的价格加 interval | 版本号 | 清单差异 |
| 来源形式 | observed、reviewed:commit、decided | observed、published_at、decided | enacted、decided | user-said |
| 谁算人 | owner | owner | 律师 | 用户本人 |

本体演化：种类和槽位在领域本体里声明，write 拒绝未声明的种类。agent 遇到本体表达不了的东西，写一条 Claim，内容是“需要一种 X 关系，因为……”，带它当时想连的两个实体。同类提案积累到人能看见时，人改本体出新版本。

---

## 6. 锚

### 6.1 锚为什么是状态机

一位“变了没有”的 diff 标记做不到、δ(state, obs, taken_at, entered_at) 能做到的事：

- **往返**：价格越过 5% 又回来，diff 会静默清零；δ 的状态保住“越过过”。
- **缺席再现**：文件删了又恢复，法条废止又重立，hash 等于基线，diff 说没变；δ 有 absent 状态，能报 returned。
- **驻留**：代码正在被编辑时每分钟都在变，记忆不该每次都被交回，该在沉降后交回；这需要 entered_at 做输入。
- **仪器变化**：读数由另一个 derivation 取得，diff 分不清世界动了还是尺子换了；状态带 derivation 身份。
- **终态**：实体退役后不能被静默重开。

所以 δ 是解释层，不是包装。它和“只有一种时效机制”相容：断言依据锚状态，锚状态由 δ 从读数序列投影出来，核心不带 TTL，但把时间交给 δ。CLAUDE.md §1 的第 1、7、8、10、13 条因此保留。

### 6.2 锚是什么

锚是传感器，(实体, 探针, δ) 三元组。一个实体可以有多个锚，各看一片现实。锚的状态是读数序列经 δ 的投影，不存；规则表内容寻址，read 只交付它的 hash；journal 只记 open、transition、revise、close，无变化的观察不是事件，不进 journal，只更新读数缓存的 (地址, hash, at)。

断言 rests_on 一个锚状态里的若干路径。stale 的判定是断言记录的状态与锚当前投影在这些路径上的比较。re-attest 换掉记录的状态，不动锚。

### 6.3 现在的实现与此设计的差距

| 差距 | 证据 |
|---|---|
| 状态被存而不是投影 | journal 里 still 46,777 条，transition 11,089；sealed 存 4,108 份状态快照；规则表随每次 read 交付 9,265 B，占 51% |
| 锚 = 坐标 = 身份 | 改名即新锚，旧锚上的一切孤立 |
| 绑定在笔记级，订阅靠手写 watch | 88% 的移动不报，22 篇无 watch |
| 领域只写了退化形状 | 16 个 dim 全是 baseline != obs；taken_at 与 entered_at 在形状里出现 0 次，求值器支持它们；没有 flux、settled、absent、returned |
| 形状放错了层 | shapes.rs 在 console/cli，packs/coding 里只有 extract 和 probes；§1 第 12 条说 pack 拥有形状 |

### 6.4 coding pack 的首批富形状

contract 形状保留现有八个轴：missing、name、file、kind、sig、surface、logic、place。在此之上加：

| 状态 | 进入条件 | 离开条件 | 用途 |
|---|---|---|---|
| flux | 任一轴在 taken_at 相对 entered_at 的驻留窗口内变过 | 驻留窗口内无变化 | 读模型可以选择在 flux 期间不交回记忆 |
| settled | flux 离开 | 任一轴变 | 交回记忆的默认时机 |
| absent | obs.found == false | obs.found == true | 依据它的断言全部 stale |
| returned | absent 离开且读数的 sig 与最后一次 present 的 sig 相同 | 下一次 transition | 报“回来了”，不是新实体 |
| replaced | absent 离开且 sig 不同 | 下一次 transition | 解析器决定是否同一 id |

state 另带 moved_count 和 last_moved_at，让往返留下证据。驻留窗口是 pack 的参数，默认由 pack 定，基座不知道它。roster、fingerprint、value 形状同理加 flux、settled、absent、returned。

---

## 7. 协议

### 7.1 wire 语法，agent 读写共用

```
@Type key [=addr] ~s      节点；=addr 是读数地址，8 位 hex
slot: text ~s             行首 ! 表示其依据在锚上变了
>rel Type:key ~s          出边；<rel 入边；缩进行是对端节点的槽位
~s who how address        来源，每种只写一次，放页脚
```

预算：一次 walk 输出上限 4 KB，信封不超过 20%，截断显式计数。程序路径用结构对象，不过文本。

### 7.2 写入

`write < doc`，同一语法，多块一文档，原子。校验顺序：类型在本体里且键形合法；事实键实体现在路由到解析器，未命中即 unresolved；槽位已声明且 required 齐全；来源对该槽位合法，Decision.claim 只接受 decided 且人；observed 必须引用本任务里 read 或 walk 发过的地址；一句话；关系种类存在且两端类型匹配；replace 的 hash 匹配。之后是噪音拒绝：历史词、日期与计量、正文里的引用、与源码重合、未解析的标识符、近重复。回复不超过 200 字节，拒绝指名行号和代码，什么都不写。

### 7.3 动词

| 动词 | 做什么 | 输出 |
|---|---|---|
| locate | 词、label 或位置到实体 | 不超过 5 行 |
| read | 一个节点，或 ontology.Type | 不超过 1 KB |
| walk | 按种类、方向、来源、stale、类型、深度取边 | 不超过 4 KB |
| write | 四个原语 | 不超过 200 字节 |
| check | 哪些断言 stale，哪些锚 transition | 安静时零输出 |

读模型是调用 read 和 walk 的外部程序。init、probes、export、import 作为操作员引导保留但隐藏。

---

## 8. 收敛

机制只有一个：做任务的 agent 顺手连边和断边，过时由锚标出。任务开始时 walk，任务结束时 write。身份稳定让更新是更新；CAS 让矛盾变成“你没读当前值”；retire 是断的原语；stale 标记是下一个 agent 的信号，不修，谁碰到谁核实。

值得看的数字，只观察不驱动：同一片代码上连续任务里，首次写入前在交付集之外读的字节数是否下降；交付的 stale 项里 re-attest、replace、retire 的分布。

---

## 9. 与现状的差距

保留：δ 与四个输入、首匹配规则表、终态、gmr-expr、fact address 与 earned derivation、探针契约、gmr-budget、ast/prose/addr 提取器、解析器。

迁移：shapes 进 packs/coding；规则表内容寻址；journal 不记 still；sealed 只留理由；memories/ 的 196 篇笔记拆成断言，714 条 Decision 与 Policy、499 个槽位值、1,049 条边是下限。

删除：Warrant、Holding、Shown、Depends 这套交付层枚举；said、ground、condense，并入 Claim 加 rests_on；sync 与 frontmatter 绑定；links 表；atlas；adopt；survey 索引；name-map；http、sql 传输；mem0、claude-code 提供者，外部记忆库不能执行槽位规则，只能做 rests_on 的目标；doctor 里依附被删机制的桶。

CLAUDE.md §1：第 9 条溶解为 rests_on 加 stale；其余保留。§2 的 memories/ 描述改为记忆图的记录；§5 新增无 IO 的 gmr-net 承载记录格式、校验器、四个原语、读入口。

---

## 10. 证伪实验，先于代码

不写 runtime，用脚本模拟行语法的 walk 和 write。选 read.rs 子系统，网初始只有从它的 38 篇笔记迁移出的 Decision，没有其他边。同一片代码上连续跑 10 个真实任务，每个任务开始时 walk 一跳，结束时 write，写入经机械校验落进网，下一个任务看到上一个留下的。对照臂用今天的 read --json 加 SKILL.md 跑同一序列。每个任务测三个数：首次编辑前在交付集之外读的字节，测试通过率，写入里无提示即合法的比例。

证伪条件：子图臂的外部读取量在 10 个任务里不下降，或合法写入不到一半。
