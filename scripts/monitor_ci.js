import { execSync } from 'node:child_process';

const REPO = 'NotToxel/BeRealStudio';
const BRANCH = process.argv[2] || 'release/v2.2.0';

console.log(`\n🔍 Monitoring GitHub Actions CI Pipeline for branch [${BRANCH}] on [${REPO}]...\n`);

async function fetchJSON(url) {
  const res = await fetch(url, {
    headers: {
      'User-Agent': 'BeRealStudio-CI-Monitor',
      'Accept': 'application/vnd.github.v3+json'
    }
  });
  if (!res.ok) {
    throw new Error(`GitHub API HTTP ${res.status}: ${res.statusText}`);
  }
  return res.json();
}

async function getLatestRunForBranch(branch) {
  const data = await fetchJSON(`https://api.github.com/repos/${REPO}/actions/runs?branch=${encodeURIComponent(branch)}`);
  return data.workflow_runs?.[0] || null;
}

async function getJobsForRun(runId) {
  const data = await fetchJSON(`https://api.github.com/repos/${REPO}/actions/runs/${runId}/jobs`);
  return data.jobs || [];
}

async function monitor() {
  const startTime = Date.now();
  
  while (true) {
    try {
      const run = await getLatestRunForBranch(BRANCH);
      if (!run) {
        console.log(`[${new Date().toLocaleTimeString()}] No workflow runs found for branch ${BRANCH}. Waiting...`);
        await new Promise(r => setTimeout(r, 10000));
        continue;
      }

      const jobs = await getJobsForRun(run.id);
      console.log(`\n============================================================`);
      console.log(`Workflow Run #${run.id} (${run.name}) | Status: ${run.status.toUpperCase()} | Conclusion: ${run.conclusion || 'RUNNING'}`);
      console.log(`HTML URL: ${run.html_url}`);
      console.log(`------------------------------------------------------------`);

      let allDone = true;
      let anyFailed = false;

      for (const job of jobs) {
        const symbol = job.status === 'completed' 
          ? (job.conclusion === 'success' ? '✔' : '✖') 
          : '⏳';
        console.log(`  ${symbol} [${job.name}] - Status: ${job.status} (${job.conclusion || 'running'})`);
        if (job.status !== 'completed') {
          allDone = false;
        }
        if (job.conclusion === 'failure' || job.conclusion === 'cancelled') {
          anyFailed = true;
        }
      }

      if (run.status === 'completed') {
        if (run.conclusion === 'success') {
          console.log(`\n🎉 ALL GITHUB ACTIONS CI CHECKS AND BUILDS PASSED SUCCESSFULLY!`);
          process.exit(0);
        } else {
          console.error(`\n❌ WORKFLOW COMPLETED WITH FAILURE: ${run.conclusion}`);
          
          // Detail failing steps
          for (const job of jobs) {
            if (job.conclusion === 'failure') {
              console.error(`\n✖ Failing Job: ${job.name} (ID: ${job.id})`);
              for (const step of job.steps || []) {
                if (step.conclusion === 'failure') {
                  console.error(`    ↳ Failed Step: ${step.name}`);
                }
              }
            }
          }
          process.exit(1);
        }
      }

      const elapsedSec = Math.round((Date.now() - startTime) / 1000);
      console.log(`\nPipeline still running (Elapsed: ${elapsedSec}s). Next check in 15s...`);
      await new Promise(r => setTimeout(r, 15000));
    } catch (e) {
      console.warn(`Warning fetching status: ${e.message}`);
      await new Promise(r => setTimeout(r, 15000));
    }
  }
}

monitor();
