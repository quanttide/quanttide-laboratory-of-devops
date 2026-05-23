const invoke = window.__TAURI__ ? window.__TAURI__.invoke : null;

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

async function scan() {
  const path = getRepoPath();
  const tbody = document.getElementById('submodules-body');
  const issues = document.getElementById('issues');
  tbody.innerHTML = '<tr><td colspan="4" class="empty">扫描中...</td></tr>';
  issues.innerHTML = '';

  if (!invoke) {
    tbody.innerHTML = '<tr><td colspan="4" class="empty">Tauri 环境未就绪（请在 Tauri 中运行）</td></tr>';
    return;
  }

  try {
    const submodules = await invoke('scan_repo', { path });
    const health = await invoke('health_check', { path });

    if (submodules.length === 0) {
      tbody.innerHTML = '<tr><td colspan="4" class="empty">没有子模块</td></tr>';
    } else {
      tbody.innerHTML = submodules.map(sm => `
        <tr onclick="showDetail('${sm.name}', '${sm.parent_pointer}', '${sm.local_head}', '${sm.remote_head}', '${sm.status}', '${sm.tracked_branch}')">
          <td>${sm.name}</td>
          <td><span class="status-dot dot-${statusClass(sm.status)}"></span>${statusLabel(sm.status)}</td>
          <td>${sm.tracked_branch}</td>
          <td>${actionButtons(sm.name, sm.status)}</td>
        </tr>
      `).join('');
    }

    // Show health issues
    if (health.length > 0) {
      issues.innerHTML = health.map(h => `
        <div class="issue ${h.status === 'Orphaned' || h.status === 'Detached' ? 'error' : h.status === 'Dirty' ? 'warning' : 'info'}">
          <strong>[${h.submodule_name}]</strong> ${h.description} — ${h.suggested_action}
        </div>
      `).join('');
    }

    // Update stats
    const clean = submodules.filter(s => s.status === 'Clean').length;
    const attention = submodules.length - clean;
    document.getElementById('stat-total').textContent = submodules.length;
    document.getElementById('stat-clean').textContent = clean;
    document.getElementById('stat-attention').textContent = attention;

    log(`扫描完成: ${submodules.length} 个子模块`);
    loadHistory();
  } catch (err) {
    tbody.innerHTML = `<tr><td colspan="4" class="empty">错误: ${err}</td></tr>`;
    log(`扫描失败: ${err}`);
  }
}

function showDetail(name, pp, local, remote, status, branch) {
  const detail = document.getElementById('detail');
  detail.style.display = 'block';
  detail.innerHTML = `
    <h3>${name} <span class="status-dot dot-${statusClass(status)}"></span>${statusLabel(status)}</h3>
    <p><strong>跟踪分支:</strong> ${branch}</p>
    <div class="commit-grid">
      <div class="commit-box">
        <div class="label">父仓库指针</div>
        <div class="hash">${pp}</div>
      </div>
      <div class="commit-box">
        <div class="label">本地 HEAD</div>
        <div class="hash">${local}</div>
      </div>
      <div class="commit-box">
        <div class="label">远程 HEAD</div>
        <div class="hash">${remote}</div>
      </div>
    </div>
    <button class="btn-sm primary" onclick="updateOne('${name}')">更新</button>
    <button class="btn-sm primary" onclick="syncOne('${name}')">同步</button>
    <button class="btn-sm danger" onclick="retireOne('${name}')">退役</button>
  `;
}

async function updateOne(name) {
  if (!invoke) return;
  try {
    const result = await invoke('update_single', { repo: getRepoPath(), name, strategy: 'fast-forward' });
    log(result);
    scan();
  } catch (err) { log(`错误: ${err}`); }
}

async function syncOne(name) {
  if (!invoke) return;
  try {
    const result = await invoke('sync_to_parent', { repo: getRepoPath(), name });
    log(result);
    scan();
  } catch (err) { log(`错误: ${err}`); }
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

async function batchUpdate() {
  if (!invoke) return;
  try {
    const result = await invoke('update_all', { path: getRepoPath(), strategy: 'fast-forward' });
    log(result);
    scan();
  } catch (err) { log(`错误: ${err}`); }
}

async function batchSync() {
  if (!invoke) return;
  try {
    const result = await invoke('sync_all_to_parent', { path: getRepoPath() });
    log(result);
    scan();
  } catch (err) { log(`错误: ${err}`); }
}

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
  if (status === 'BehindRemote' || status === 'Uninitialized') btns += `<button class="btn-sm primary" onclick="updateOne('${name}')">更新</button>`;
  if (status === 'AheadOfParent') btns += `<button class="btn-sm primary" onclick="syncOne('${name}')">同步</button>`;
  if (status === 'Dirty') btns += `<button class="btn-sm primary" onclick="updateOne('${name}')">提交</button>`;
  if (status !== 'Clean') btns += `<button class="btn-sm danger" onclick="retireOne('${name}')">退役</button>`;
  return btns;
}

async function loadHistory() {
  if (!invoke) return;
  const el = document.getElementById('history-list');
  try {
    const records = await invoke('list_history', { path: getRepoPath(), limit: 10, submodule: null });
    if (records.length === 0) {
      el.innerHTML = '<div class="msg">暂无操作记录</div>';
    } else {
      el.innerHTML = records.map(r =>
        `<div class="msg">${r.success ? '✓' : '✗'} ${r.timestamp} ${r.action}: ${r.submodule_name}</div>`
      ).join('');
    }
  } catch (e) {
    el.innerHTML = `<div class="msg">加载历史失败: ${e}</div>`;
  }
}

// Auto-scan on load with debounce
let scanTimer;
document.getElementById('repo-path').addEventListener('input', () => {
  clearTimeout(scanTimer);
  scanTimer = setTimeout(scan, 500);
});

document.addEventListener('DOMContentLoaded', scan);
