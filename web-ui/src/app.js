const invoke = window.__TAURI__ ? window.__TAURI__.invoke : null;

// ---- Modal (dry-run preview) ----
let pendingAction = null;

function showModal(title, body, onConfirm) {
  document.getElementById('modal-title').textContent = title;
  document.getElementById('modal-body').textContent = body;
  document.getElementById('modal-confirm').onclick = () => { closeModal(); if (onConfirm) onConfirm(); };
  document.getElementById('modal-overlay').style.display = 'flex';
}

function closeModal() {
  document.getElementById('modal-overlay').style.display = 'none';
  pendingAction = null;
}

// ---- Progress ----
function showProgress(msg) {
  const el = document.getElementById('progress');
  el.textContent = msg;
  el.style.display = 'block';
}

function hideProgress() {
  document.getElementById('progress').style.display = 'none';
}

// ---- Messaging ----
function log(msg) {
  const el = document.getElementById('messages');
  const div = document.createElement('div');
  div.className = 'msg';
  div.textContent = msg;
  el.appendChild(div);
  el.scrollTop = el.scrollHeight;
}

function getRepoPath() {
  return document.getElementById('repo-path').value.trim() || '.';
}

// ---- Selection ----
function toggleSelectAll() {
  const checked = document.getElementById('select-all').checked;
  document.querySelectorAll('.sm-checkbox').forEach(cb => cb.checked = checked);
}

function getSelectedNames() {
  return Array.from(document.querySelectorAll('.sm-checkbox:checked'))
    .map(cb => cb.value)
    .filter(Boolean);
}

// ---- Scan ----
async function scan() {
  const path = getRepoPath();
  const tbody = document.getElementById('submodules-body');
  const issues = document.getElementById('issues');
  tbody.innerHTML = '<tr><td colspan="5" class="empty">扫描中...</td></tr>';
  issues.innerHTML = '';
  hideProgress();

  if (!invoke) {
    tbody.innerHTML = '<tr><td colspan="5" class="empty">Tauri 环境未就绪（请在 Tauri 中运行）</td></tr>';
    return;
  }

  try {
    const submodules = await invoke('scan_repo', { path });
    const health = await invoke('health_check', { path });

    if (submodules.length === 0) {
      tbody.innerHTML = '<tr><td colspan="5" class="empty">没有子模块</td></tr>';
    } else {
      tbody.innerHTML = submodules.map(sm => `
        <tr onclick="showDetail('${sm.name}', '${sm.parent_pointer}', '${sm.local_head}', '${sm.remote_head}', '${sm.status}', '${sm.tracked_branch}', ${sm.ahead_count || 0}, ${sm.behind_count || 0})">
          <td onclick="event.stopPropagation()"><input type="checkbox" class="sm-checkbox" value="${sm.name}" /></td>
          <td>${sm.name}</td>
          <td><span class="status-dot dot-${statusClass(sm.status)}"></span>${statusLabel(sm.status)}</td>
          <td>${sm.tracked_branch}</td>
          <td>${actionButtons(sm.name, sm.status)}</td>
        </tr>
      `).join('');
    }

    if (health.length > 0) {
      issues.innerHTML = health.map(h => `
        <div class="issue ${h.status === 'Orphaned' || h.status === 'Detached' ? 'error' : h.status === 'Dirty' ? 'warning' : 'info'}">
          <strong>[${h.submodule_name}]</strong> ${h.description} — ${h.suggested_action}
        </div>
      `).join('');
    }

    const clean = submodules.filter(s => s.status === 'Clean').length;
    const attention = submodules.length - clean;
    document.getElementById('stat-total').textContent = submodules.length;
    document.getElementById('stat-clean').textContent = clean;
    document.getElementById('stat-attention').textContent = attention;

    log(`扫描完成: ${submodules.length} 个子模块`);
    loadHistory();
  } catch (err) {
    tbody.innerHTML = `<tr><td colspan="5" class="empty">错误: ${err}</td></tr>`;
    log(`扫描失败: ${err}`);
  }
}

// ---- Detail ----
function showDetail(name, pp, local, remote, status, branch, ahead, behind) {
  const detail = document.getElementById('detail');
  detail.style.display = 'block';
  let diffHtml = '';
  if (ahead > 0 && behind > 0) diffHtml = `<p>差异: +${ahead} / -${behind}</p>`;
  else if (ahead > 0) diffHtml = `<p>差异: 领先 <strong>+${ahead}</strong></p>`;
  else if (behind > 0) diffHtml = `<p>差异: 落后 <strong>-${behind}</strong></p>`;
  else diffHtml = `<p>差异: 同步</p>`;

  let guidance = '';
  let fixButtons = '';
  switch (status) {
    case 'Dirty':
      guidance = '<p class="status-dirty">有未提交的修改。建议: 手动 commit 或 stash。</p>';
      fixButtons = `<button class="btn-sm primary" onclick="alert('请在子模块目录中手动执行 git status 查看变更')">查看变更</button>`;
      break;
    case 'Detached':
      guidance = '<p class="status-dirty">游离 HEAD 状态。建议: 切换到跟踪分支。</p>';
      fixButtons = `<button class="btn-sm primary" onclick="updateOne('${name}')">修复: 切换分支</button>`;
      break;
    case 'BehindRemote':
      guidance = '<p>远程有更新。建议: 执行更新。</p>';
      fixButtons = `<button class="btn-sm primary" onclick="updateOne('${name}')">修复: 更新</button>`;
      break;
    case 'AheadOfParent':
      guidance = '<p>本地领先于父仓库记录。建议: 同步到父仓库。</p>';
      fixButtons = `<button class="btn-sm primary" onclick="syncOne('${name}')">修复: 同步</button>`;
      break;
    case 'Uninitialized':
      guidance = '<p>尚未初始化。建议: 初始化子模块。</p>';
      fixButtons = `<button class="btn-sm primary" onclick="updateOne('${name}')">修复: 初始化</button>`;
      break;
    case 'Orphaned':
      guidance = '<p class="status-dirty">父仓库记录的 commit 在远程已不存在。需手动干预。</p>';
      break;
    default:
      guidance = '<p>状态正常，无需操作。</p>';
  }

  detail.innerHTML = `
    <h3>${name} <span class="status-dot dot-${statusClass(status)}"></span>${statusLabel(status)}</h3>
    <p><strong>跟踪分支:</strong> ${branch}</p>
    ${diffHtml}
    ${guidance}
    <div class="commit-grid">
      <div class="commit-box"><div class="label">父仓库指针</div><div class="hash">${pp}</div></div>
      <div class="commit-box"><div class="label">本地 HEAD</div><div class="hash">${local}</div></div>
      <div class="commit-box"><div class="label">远程 HEAD</div><div class="hash">${remote}</div></div>
    </div>
    <div style="margin-top:8px;display:flex;gap:6px;flex-wrap:wrap">
      ${fixButtons}
      <button class="btn-sm primary" onclick="updateOne('${name}')">更新</button>
      <button class="btn-sm primary" onclick="syncOne('${name}')">同步</button>
      <button class="btn-sm danger" onclick="retireOne('${name}')">退役</button>
    </div>
  `;
}

// ---- Single actions with dry-run modal ----
async function confirmAndRun(title, description, actionFn) {
  showModal(title, description + '\n\n确认执行？', actionFn);
}

async function updateOne(name) {
  if (!invoke) return;
  confirmAndRun('更新子模块', `更新子模块: ${name}\n策略: fast-forward`, async () => {
    try {
      const result = await invoke('update_single', { repo: getRepoPath(), name, strategy: 'fast-forward' });
      log(result);
      scan();
    } catch (err) { log(`错误: ${err}`); }
  });
}

async function syncOne(name) {
  if (!invoke) return;
  confirmAndRun('同步子模块', `同步子模块: ${name}\n将更新父仓库 commit 指针`, async () => {
    try {
      const result = await invoke('sync_to_parent', { repo: getRepoPath(), name });
      log(result);
      scan();
    } catch (err) { log(`错误: ${err}`); }
  });
}

async function retireOne(name) {
  if (!confirm(`确定退役子模块 "${name}"？`)) return;
  if (!invoke) return;
  try {
    const result = await invoke('retire_submodule', { repo: getRepoPath(), name });
    log(result);
    scan();
  } catch (err) { log(`错误: ${err}`); }
}

// ---- Batch actions ----
async function execOnSelected(actionLabel, tauriCmd, paramKey) {
  const names = getSelectedNames();
  if (names.length === 0) { log('请先勾选要操作的子模块'); return; }
  const preview = names.map(n => `  ${n}`).join('\n');
  const desc = `${actionLabel}\n\n选中 ${names.length} 个子模块:\n${preview}`;
  showModal(actionLabel, desc, async () => {
    showProgress(`正在执行 ${actionLabel}... (0/${names.length})`);
    let done = 0, failed = 0;
    for (const name of names) {
      try {
        await invoke(tauriCmd, { repo: getRepoPath(), name, [paramKey || 'name']: name });
        done++;
      } catch (err) {
        log(`错误: ${name}: ${err}`);
        failed++;
      }
      showProgress(`正在执行 ${actionLabel}... (${done + failed}/${names.length})`);
    }
    hideProgress();
    log(`${actionLabel} 完成: ${done} 成功, ${failed} 失败`);
    scan();
  });
}

async function batchUpdateSelected() {
  execOnSelected('更新选中子模块', 'update_single', 'strategy');
}

async function batchSyncSelected() {
  execOnSelected('同步选中子模块', 'sync_to_parent');
}

async function batchUpdate() {
  showModal('全部更新', '将所有子模块更新到最新版本\n策略: fast-forward', async () => {
    try {
      const result = await invoke('update_all', { path: getRepoPath(), strategy: 'fast-forward' });
      log(result);
      scan();
    } catch (err) { log(`错误: ${err}`); }
  });
}

async function batchSync() {
  showModal('全部同步', '将所有子模块指针同步到父仓库', async () => {
    try {
      const result = await invoke('sync_all_to_parent', { path: getRepoPath() });
      log(result);
      scan();
    } catch (err) { log(`错误: ${err}`); }
  });
}

// ---- CI export ----
async function exportCI(format) {
  if (!invoke) return;
  try {
    const script = await invoke('export_ci', { path: getRepoPath(), format });
    await navigator.clipboard.writeText(script);
    log(`已复制 ${format} CI 脚本到剪贴板`);
  } catch (err) { log(`导出失败: ${err}`); }
}

// ---- History ----
async function loadHistory() {
  if (!invoke) return;
  const el = document.getElementById('history-list');
  const start = document.getElementById('hist-start').value || null;
  const end = document.getElementById('hist-end').value || null;
  try {
    const records = await invoke('list_history', { path: getRepoPath(), limit: 10, submodule: null, startDate: start, endDate: end });
    if (records.length === 0) {
      el.innerHTML = '<div class="msg">暂无操作记录</div>';
    } else {
      el.innerHTML = records.map(r =>
        `<div class="msg">${r.success ? '✓' : '✗'} ${r.timestamp} ${r.action}: ${r.submodule_name}</div>`
      ).join('');
    }
  } catch (e) {
    el.innerHTML = `<div class="msg">加载失败: ${e}</div>`;
  }
}

// ---- Helpers ----
function statusClass(status) {
  switch (status) {
    case 'Clean': return 'clean';
    case 'AheadOfParent': case 'BehindRemote': return 'ahead';
    case 'Detached': case 'Dirty': case 'Orphaned': return 'detached';
    case 'Uninitialized': return 'uninitialized';
    default: return 'uninitialized';
  }
}

function statusLabel(status) {
  switch (status) {
    case 'Clean': return '干净';
    case 'AheadOfParent': return '领先';
    case 'BehindRemote': return '落后';
    case 'Detached': return '游离';
    case 'Dirty': return '脏';
    case 'Orphaned': return '孤儿';
    case 'Uninitialized': return '未初始化';
    default: return status;
  }
}

function actionButtons(name, status) {
  if (status === 'Clean') return '';
  let btns = '';
  if (status === 'BehindRemote' || status === 'Uninitialized') btns += `<button class="btn-sm primary" onclick="event.stopPropagation();updateOne('${name}')">更新</button>`;
  if (status === 'AheadOfParent') btns += `<button class="btn-sm primary" onclick="event.stopPropagation();syncOne('${name}')">同步</button>`;
  if (status === 'Dirty') btns += `<button class="btn-sm primary" onclick="event.stopPropagation();alert('请手动执行 git status')">查看</button>`;
  if (status !== 'Clean') btns += `<button class="btn-sm danger" onclick="event.stopPropagation();retireOne('${name}')">退役</button>`;
  return btns;
}

// ---- Init ----
let scanTimer;
document.getElementById('repo-path').addEventListener('input', () => {
  clearTimeout(scanTimer);
  scanTimer = setTimeout(scan, 500);
});

document.addEventListener('DOMContentLoaded', scan);
