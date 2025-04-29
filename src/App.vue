<template>
  <div id="app">
    <h1>PHP TinkerLab</h1>
    <input class="pathInput" v-model="path" placeholder="Enter the path to your project..." />
    <div class="flex">
      <div class="textarea">
        <textarea v-model="phpCode" placeholder="Enter PHP code..." rows="5"></textarea>
        <button class="runCode" @click="runTinkerCode">Run Code</button>
      </div>
      <div class="results">
        <div class="result" v-if="result">
          <pre>{{ result }}</pre>
        </div>
        <div v-if="error" class="error">
          <h3>Error:</h3>
          <pre>{{ error }}</pre>
        </div>
      </div>
    </div>
  
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const phpCode = ref('');
const path = ref('laravel/path/to/project');
const result = ref('');
const error = ref('');

const runTinkerCode = async () => {
  try {
    // Call the backend to run the PHP code in tinker
    result.value = await invoke('run_php_in_tinker', { path: path.value, code: phpCode.value });
    error.value = ''; // Clear error if successful
  } catch (err) {
    result.value = ''; // Clear result on error
    error.value = `Error: ${err}`;
  }
};
</script>

<style>

* {
  box-sizing: border-box;
}

.pathInput {
  width: 100%;
  padding: 10px;
  margin-bottom: 10px;
  border: 1px solid #fff;
  background: #1a1a1a;
  color: #FFF;
}

.flex {
  display: flex;
}

.textarea {
  width: 100%;
  height: calc( 100vh - 200px );
}

.textarea textarea {
  width: 100%;
  height: 100vh;
  background: #1a1a1a;
  color: #FFF;
  border: none;
  padding: 10px;
  border: 1px solid #fff;
  appearance: none;
  resize: none;
  width: 100%;
  height: 100%;
}

.results {
  width: 100%;
  height: calc( 100vh - 200px );
  border: 1px solid #fff;
  border-left: none;
  overflow: hidden;
}

.results .result {
  overflow: hidden;
  padding: 10px;
  width: 100%;
  color: #FFF;
}


h1 {
  color: #FFF;
}

html {
  background: #1a1a1a;
}

.error {
  color: red;
  font-weight: bold;
}

.runCode {
  background: #1a1a1a;
  color: #FFF;
  border: 1px solid #fff;
  padding: 10px;
  cursor: pointer;
}
.runCode:hover {
  background: #fff;
  color: #1a1a1a;
}
</style>
