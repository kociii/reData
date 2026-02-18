# reData 后端功能清单 (Backend API Todolist)

## 一、项目管理 (`/api/projects/`)

- [x] 1. 查询项目列表 — `GET /`
- [x] 2. 新建项目 — `POST /`（自动创建项目数据表）
- [x] 3. 查询项目详情 — `GET /{project_id}`
- [x] 4. 更新项目 — `PUT /{project_id}`（名称、描述、去重配置）
- [x] 5. 删除项目 — `DELETE /{project_id}`（级联删除字段+数据表）

## 二、字段管理 (`/api/fields/`)

- [x] 6. 添加项目字段 — `POST /`（自动同步数据表结构）
- [x] 7. 编辑项目字段 — `PUT /{field_id}`（自动同步表结构）
- [x] 8. 删除项目字段 — `DELETE /{field_id}`（软删除，标记 is_deleted）
- [x] 9. 恢复已删除字段 — `POST /{field_id}/restore`
- [x] 10. 查询项目字段列表 — `GET /project/{project_id}`（过滤已删除）
- [x] 11. 查询全部字段（含已删除） — `GET /project/{project_id}/all`
- [x] 12. AI 辅助生成字段元数据 — `POST /generate-metadata`（翻译英文名、生成验证规则、提取提示）

## 三、AI 配置管理 (`/api/ai-configs/`)

- [x] 13. 新增 AI 配置 — `POST /`
- [x] 14. 查询 AI 配置列表 — `GET /`
- [x] 15. 查询单个 AI 配置 — `GET /{config_id}`
- [x] 16. 查询默认 AI 配置 — `GET /default`
- [x] 17. 更新 AI 配置 — `PUT /{config_id}`
- [x] 18. 删除 AI 配置 — `DELETE /{config_id}`
- [x] 19. 测试 AI 连接 — `POST /test-connection`
- [x] 20. 设置默认配置 — `POST /{config_id}/set-default`

## 四、文件管理 (`/api/files/`)

- [x] 21. 上传单个文件 — `POST /upload`
- [x] 22. 批量上传文件 — `POST /upload-multiple`
- [x] 23. 预览文件内容 — `GET /preview/{file_id}`（前 10 行，支持指定 Sheet）
- [x] 24. 获取文件信息 — `GET /info/{file_id}`（大小、Sheet 列表）
- [x] 25. 删除临时文件 — `DELETE /{file_id}`
- [x] 26. 获取批次文件列表 — `GET /batch/{batch_number}`
- [x] 27. 下载文件 — `GET /download/{file_id}`
- [x] 28. 清理临时文件 — `POST /cleanup`

## 五、数据处理 (`/api/processing/`)

- [x] 29. 启动处理任务 — `POST /start`（返回 task_id + batch_number）
- [x] 30. 暂停任务 — `POST /pause/{task_id}`
- [x] 31. 恢复任务 — `POST /resume/{task_id}`
- [x] 32. 取消任务 — `POST /cancel/{task_id}`
- [x] 33. 查询任务状态 — `GET /status/{task_id}`
- [x] 34. 查询项目任务列表 — `GET /list/{project_id}`（支持状态筛选、分页）
- [x] 35. WebSocket 实时进度推送 — `WS /ws/progress/{task_id}`

### 两阶段处理核心流程（服务层）

- [x] 36. 阶段一：AI 列映射分析 — 读取前 10 行样本，AI 识别表头+列映射（每 Sheet 仅 1 次 AI 调用）
- [x] 37. 阶段二：本地验证导入 — 根据映射读取列数据，格式验证，逐行插入（无 AI 调用）
- [x] 38. 去重处理 — 支持 skip/update/merge 三种策略
- [x] 39. 数据格式验证 — 手机号/邮箱/URL/日期/数字 正则验证
- [x] 40. 数据标准化 — 手机号去前缀、邮箱转小写、日期统一格式
- [x] 41. 空行检测 — 连续 10 行空行自动跳到下一个 Sheet
- [x] 42. 多 Sheet 处理 — 每个 Sheet 独立表头识别和列映射
- [x] 43. 批次目录管理 — 文件复制到 `history/batch_XXX/`，保留原始文件

## 六、结果管理 (`/api/results/`)

- [x] 44. 查询结果列表 — `GET /{project_id}`（分页、批次筛选、状态筛选、关键词搜索、排序）
- [x] 45. 查询单条记录 — `GET /{project_id}/{record_id}`
- [x] 46. 编辑记录 — `PUT /{project_id}/{record_id}`
- [x] 47. 删除记录 — `DELETE /{project_id}/{record_id}`
- [x] 48. 删除批次记录 — `DELETE /{project_id}/batch/{batch_number}`
- [x] 49. 导出结果 — `GET /export/{project_id}`（支持 xlsx/csv，支持批次筛选）
- [x] 50. 项目统计信息 — `GET /statistics/{project_id}`（总记录数、成功/失败数、批次数）

## 七、基础设施

- [x] 51. 健康检查 — `GET /health`
- [x] 52. 根路由信息 — `GET /`
- [x] 53. CORS 跨域配置 — 支持 Tauri 前端访问
- [x] 54. 数据库自动初始化 — 启动时自动创建表
- [x] 55. 动态表结构管理 — 根据字段定义动态创建/迁移项目数据表
- [x] 56. 智能表结构迁移 — 添加/删除字段时尽可能保留数据
