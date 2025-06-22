(function(){const e=document.createElement("link").relList;if(e&&e.supports&&e.supports("modulepreload"))return;for(const s of document.querySelectorAll('link[rel="modulepreload"]'))o(s);new MutationObserver(s=>{for(const r of s)if(r.type==="childList")for(const a of r.addedNodes)a.tagName==="LINK"&&a.rel==="modulepreload"&&o(a)}).observe(document,{childList:!0,subtree:!0});function t(s){const r={};return s.integrity&&(r.integrity=s.integrity),s.referrerPolicy&&(r.referrerPolicy=s.referrerPolicy),s.crossOrigin==="use-credentials"?r.credentials="include":s.crossOrigin==="anonymous"?r.credentials="omit":r.credentials="same-origin",r}function o(s){if(s.ep)return;s.ep=!0;const r=t(s);fetch(s.href,r)}})();const P="modulepreload",$=function(i){return"/mammut-fft/"+i},g={},F=function(e,t,o){let s=Promise.resolve();if(t&&t.length>0){let a=function(c){return Promise.all(c.map(d=>Promise.resolve(d).then(p=>({status:"fulfilled",value:p}),p=>({status:"rejected",reason:p}))))};document.getElementsByTagName("link");const n=document.querySelector("meta[property=csp-nonce]"),l=n?.nonce||n?.getAttribute("nonce");s=a(t.map(c=>{if(c=$(c),c in g)return;g[c]=!0;const d=c.endsWith(".css"),p=d?'[rel="stylesheet"]':"";if(document.querySelector(`link[href="${c}"]${p}`))return;const h=document.createElement("link");if(h.rel=d?"stylesheet":P,d||(h.as="script"),h.crossOrigin="",h.href=c,l&&h.setAttribute("nonce",l),document.head.appendChild(h),d)return new Promise((f,m)=>{h.addEventListener("load",f),h.addEventListener("error",()=>m(new Error(`Unable to preload CSS for ${c}`)))})}))}function r(a){const n=new Event("vite:preloadError",{cancelable:!0});if(n.payload=a,window.dispatchEvent(n),!n.defaultPrevented)throw a}return s.then(a=>{for(const n of a||[])n.status==="rejected"&&r(n.reason);return e().catch(r)})};class x{static wasmModule=null;static initPromise=null;static async initialize(){if(this.initPromise)return this.initPromise;this.initPromise=this.loadWasm(),await this.initPromise}static async loadWasm(){const{default:e,WasmAudioProcessor:t}=await F(async()=>{const{default:o,WasmAudioProcessor:s}=await import("./mammut_fft_web-C1hWMWDU.js");return{default:o,WasmAudioProcessor:s}},[]);await e(),this.wasmModule={WasmAudioProcessor:t}}static createProcessor(){if(!this.wasmModule)throw new Error("WASM module not initialized");return new this.wasmModule.WasmAudioProcessor}}class q extends EventTarget{processor;audioContext=null;currentFileName="";hasUnprocessedOperations=!1;constructor(){super(),this.processor=x.createProcessor()}async loadAudioFile(e,t){this.currentFileName=e.name,this.hasUnprocessedOperations=!1,this.audioContext||(this.audioContext=new AudioContext);const o=await e.arrayBuffer(),s=new Uint8Array(o);try{this.processor.read_audio_bytes(s,t),this.dispatchEvent(new CustomEvent("audioLoaded",{detail:{fileName:e.name,info:this.getInfo()}}))}catch(r){throw new Error(`Failed to load audio: ${r}`)}}getInfo(){return this.processor.get_info()}getSpectrumData(e,t,o){return this.processor.get_spectrum_data(e,t,o)}async processAudio(){this.processor.perform_ifft(),this.hasUnprocessedOperations=!1;const e=this.processor.get_processed_audio_normalized();if(!this.audioContext)throw new Error("Audio context not initialized");const t=this.getInfo(),o=t.channels,s=e.length/o,r=this.audioContext.createBuffer(o,s,t.sample_rate);for(let a=0;a<o;a++){const n=r.getChannelData(a);for(let l=0;l<s;l++)n[l]=e[l*o+a]}return this.audioBufferToWav(r)}downloadProcessedAudio(e){let t=!1;this.hasUnprocessedOperations&&(console.log("Audio operations detected - performing inverse FFT before download..."),this.processor.perform_ifft(),this.hasUnprocessedOperations=!1,t=!0);const o=this.processor.save_to_wav_bytes(),s=new Blob([o],{type:"audio/wav"}),r=URL.createObjectURL(s),a=e||(this.currentFileName?`${this.currentFileName.replace(/\.[^/.]+$/,"")}_processed.wav`:"processed_audio.wav"),n=document.createElement("a");return n.href=r,n.download=a,document.body.appendChild(n),n.click(),document.body.removeChild(n),URL.revokeObjectURL(r),{wasProcessed:t}}reset(){this.processor.reset(),this.hasUnprocessedOperations=!1,this.dispatchEvent(new Event("reset"))}applyPower(e){this.processor.apply_pow(e),this.dispatchEvent(new Event("operationApplied"))}applyLowpass(e){this.processor.apply_lowpass(e),this.dispatchEvent(new Event("operationApplied"))}applyHighpass(e){this.processor.apply_highpass(e),this.dispatchEvent(new Event("operationApplied"))}applyBandpass(e,t){this.processor.apply_bandpass(e,t),this.dispatchEvent(new Event("operationApplied"))}audioBufferToWav(e){const t=e.numberOfChannels,o=e.length*t*2,s=e.sampleRate,r=new ArrayBuffer(44+o),a=new DataView(r),n=(c,d)=>{for(let p=0;p<d.length;p++)a.setUint8(c+p,d.charCodeAt(p))};n(0,"RIFF"),a.setUint32(4,36+o,!0),n(8,"WAVE"),n(12,"fmt "),a.setUint32(16,16,!0),a.setUint16(20,1,!0),a.setUint16(22,t,!0),a.setUint32(24,s,!0),a.setUint32(28,s*t*2,!0),a.setUint16(32,t*2,!0),a.setUint16(34,16,!0),n(36,"data"),a.setUint32(40,o,!0);let l=44;for(let c=0;c<e.length;c++)for(let d=0;d<t;d++){const p=Math.max(-1,Math.min(1,e.getChannelData(d)[c])),h=p<0?p*32768:p*32767;a.setInt16(l,h,!0),l+=2}return r}destroy(){this.processor.free()}dispatchOperationApplied(){this.hasUnprocessedOperations=!0,this.dispatchEvent(new Event("operationApplied"))}applyPhaseMultiply(e){this.processor.apply_phase_multiply(e),this.dispatchOperationApplied()}applySpectrumShift(e){this.processor.apply_spectrum_shift(e),this.dispatchOperationApplied()}applyStretch(e){this.processor.apply_stretch(e),this.dispatchEvent(new Event("operationApplied"))}applyWobble(e,t){this.processor.apply_wobble(e,t),this.dispatchOperationApplied()}applyThreshold(e,t){this.processor.apply_threshold(e,t),this.dispatchOperationApplied()}applyAmplitudeDerivative(e){this.processor.apply_amplitude_derivative(e),this.dispatchOperationApplied()}keepPeaks(){this.processor.keep_peaks(),this.dispatchOperationApplied()}swapBins(e,t){this.processor.swap_bins(e,t),this.dispatchOperationApplied()}swapChannels(e){this.processor.swap_channels(e),this.dispatchOperationApplied()}applyChordFilter(e,t,o,s){if(e.length!==5||t.length!==5)throw new Error("Chord filter requires exactly 5 frequencies and amplitudes");this.processor.apply_chord_filter(e[0],t[0],e[1],t[1],e[2],t[2],e[3],t[3],e[4],t[4],o,s),this.dispatchOperationApplied()}prepareSplitPart(e,t,o,s){this.processor.prepare_split_part(e,t,o,s)}resetSplit(){this.processor.reset_split()}async convolveWithFile(e,t,o){this.processor.convolve_with_file(e,t,o),this.dispatchOperationApplied()}}class B extends HTMLElement{audioService;constructor(){super(),this.audioService=new q}connectedCallback(){this.render(),this.setupEventListeners()}disconnectedCallback(){this.audioService.destroy()}render(){this.innerHTML=`
      <div class="container">
        <div class="header">
          <img src="/mammut-fft/mammut-fft-logo.png" alt="Mammut FFT" width="706" height="123">
        </div>
                
        <file-uploader></file-uploader>
        
        <status-bar></status-bar>        
        
        <div class="main-content" id="mainContent" style="display: none;">
          <audio-player></audio-player>
          <spectrum-visualizer></spectrum-visualizer>
          <operation-tabs></operation-tabs>
        </div>
      </div>
    `}setupEventListeners(){this.addEventListener("fileSelected",async e=>{const t=e,{file:o,bufferMultiplier:s}=t.detail;try{this.updateStatus("Loading audio file..."),await this.audioService.loadAudioFile(o,s);const r=this.querySelector("#mainContent");r&&(r.style.display="block"),this.querySelector("audio-player")?.setEnabled(!0),this.updateStatus("Audio loaded successfully!","success"),this.updateAllComponents()}catch(r){this.updateStatus(`Error loading audio: ${r}`,"error")}}),this.audioService.addEventListener("audioLoaded",async()=>{this.updateAllComponents();const e=await this.audioService.processAudio();this.querySelector("audio-player")?.updateAudio(e)}),this.audioService.addEventListener("operationApplied",()=>{this.updateStatus("Operation applied..."),this.updateSpectrum(),this.querySelector("audio-player")?.resetProcessed()}),this.audioService.addEventListener("reset",()=>{this.updateAllComponents()}),this.addEventListener("processAudio",async()=>{try{this.updateStatus("Processing audio...");const e=await this.audioService.processAudio();this.querySelector("audio-player")?.updateAudio(e),this.updateStatus("Audio processed successfully!","success")}catch(e){this.updateStatus(`Error processing audio: ${e}`,"error")}}),this.addEventListener("downloadAudio",()=>{try{this.audioService.downloadProcessedAudio().wasProcessed?this.updateStatus("Audio automatically processed and downloaded successfully!","success"):this.updateStatus("Audio downloaded successfully!","success")}catch(e){this.updateStatus(`Error downloading audio: ${e}`,"error")}}),this.addEventListener("resetAudio",async()=>{this.audioService.reset(),this.updateAllComponents(),this.updateStatus("Audio reset to original state","success");const e=await this.audioService.processAudio();this.querySelector("audio-player")?.updateAudio(e)})}updateStatus(e,t="info"){this.querySelector("status-bar")?.setStatus(e,t)}updateAllComponents(){this.updateSpectrum(),this.updateOperations()}updateSpectrum(){this.querySelector("spectrum-visualizer")?.updateSpectrum(this.audioService)}updateOperations(){this.querySelector("operation-tabs")?.setAudioService(this.audioService)}}customElements.define("audio-processor-app",B);class T extends HTMLElement{shadowRoot;fileInput;bufferMultiplierInput;constructor(){super(),this.shadowRoot=this.attachShadow({mode:"open"})}connectedCallback(){this.render(),this.setupEventListeners()}render(){this.shadowRoot.innerHTML=`
      <style>
        :host {
          display: block;
          margin: 20px 0;
        }
        
        .upload-section {
          border: 2px dashed var(--border-color, #333);
          border-radius: 8px;
          padding: 20px;
          text-align: center;
          transition: all 0.3s ease;
          background-color: rgba(255, 255, 255, 0.02);
        }
        
        .upload-section:hover {
          border-color: var(--accent-color, #4f9eff);
          background-color: rgba(255, 255, 255, 0.04);
        }
        
        .upload-section.drag-over {
          background-color: rgba(79, 158, 255, 0.1);
          border-color: var(--accent-color, #4f9eff);
          box-shadow: 0 0 10px rgba(79, 158, 255, 0.2);
        }
        
        .file-input-wrapper {
          display: flex;
          justify-content: center;
          align-items: center;
          gap: 20px;
          margin-top: 15px;
        }
        
        input[type="file"] {
          display: none;
        }
        
        .upload-button {
          padding: 12px 24px;
          background-color: var(--accent-color, #4f9eff);
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 15px;
          font-weight: 500;
          transition: background-color 0.2s;
        }
        
        .upload-button:hover {
          background-color: var(--accent-hover, #3d7dcf);
        }
        
        .buffer-control {
          display: flex;
          align-items: center;
          gap: 10px;
        }
        
        .buffer-control label {
          font-size: 14px;
          color: var(--text-secondary, #aaa);
        }
        
        .buffer-control input {
          width: 60px;
          padding: 6px;
          background-color: rgba(0, 0, 0, 0.2);
          border: 1px solid var(--border-color, #333);
          border-radius: 4px;
          color: var(--text-primary, #e0e0e0);
        }
        
        .file-info {
          margin-top: 15px;
          font-size: 14px;
          color: var(--text-secondary, #aaa);
        }
        
        .drag-message {
          display: none;
          margin: 15px 0;
          font-size: 16px;
          color: var(--accent-color, #4f9eff);
          font-weight: 500;
        }
        
        .upload-section.drag-over .drag-message {
          display: block;
        }
      </style>
      
      <div class="upload-section" id="uploadSection">
        <label>Select audio file to process or drop file here</label>
        <div class="file-input-wrapper">
          <input type="file" id="audioFile" accept="audio/*">
          <button class="upload-button" id="uploadButton">Open Audio</button>
          
          <div class="buffer-control">
            <label for="bufferMultiplier">Buffer Size:</label>
            <input type="number" id="bufferMultiplier" min="1" max="8" value="1">
          </div>
        </div>
        <div class="drag-message">Release to upload audio file</div>
        <div class="file-info" id="fileInfo"></div>
      </div>
    `,this.fileInput=this.shadowRoot.getElementById("audioFile"),this.bufferMultiplierInput=this.shadowRoot.getElementById("bufferMultiplier")}setupEventListeners(){const e=this.shadowRoot.getElementById("uploadSection");this.shadowRoot.getElementById("uploadButton").addEventListener("click",()=>{this.fileInput.click()}),this.fileInput.addEventListener("change",o=>{const s=o.target;s.files?.length&&this.handleFile(s.files[0])}),["dragenter","dragover","dragleave","drop"].forEach(o=>{e.addEventListener(o,this.preventDefaults)}),["dragenter","dragover"].forEach(o=>{e.addEventListener(o,()=>{e.classList.add("drag-over")})}),["dragleave","drop"].forEach(o=>{e.addEventListener(o,()=>{e.classList.remove("drag-over")})}),e.addEventListener("drop",o=>{const s=o.dataTransfer;s?.files.length&&this.handleFile(s.files[0])})}preventDefaults(e){e.preventDefault(),e.stopPropagation()}handleFile(e){const t=this.shadowRoot.getElementById("fileInfo"),o=(e.size/1024).toFixed(1);t.textContent=`Selected: ${e.name} (${o} KB)`;const s=parseInt(this.bufferMultiplierInput.value);this.dispatchEvent(new CustomEvent("fileSelected",{detail:{file:e,bufferMultiplier:s},bubbles:!0,composed:!0}))}}customElements.define("file-uploader",T);class M extends HTMLElement{root;audio;resetButton;downloadButton;isProcessed=!1;isProcessing=!1;constructor(){super(),this.root=this.attachShadow({mode:"open"})}connectedCallback(){this.render(),this.setupEventListeners()}render(){this.root.innerHTML=`
      <style>
        :host {
          display: block;
          margin: 25px 0;
        }
        
        .audio-panel {
          display: flex;
          justify-content: space-between;
          align-items: center;
          gap: 20px;
          background-color: rgba(255, 255, 255, 0.05);
          padding: 15px;
          border-radius: 8px;
          flex-wrap: wrap;
        }
        
        audio {
          width: 60%;
          min-width: 300px;
          height: 40px;
          background-color: rgba(0, 0, 0, 0.2);
          border-radius: 4px;
          outline: none;
        }
        
        .button-group {
          display: flex;
          gap: 10px;
          flex-wrap: wrap;
        }
        
        button {
          padding: 10px 20px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          transition: all 0.2s;
          color: white;
        }
        
        .reset-button {
          background-color: #666;
        }
        
        .reset-button:hover {
          background-color: #777;
        }
        
        .download-button {
          background-color: var(--accent-color, #4f9eff);
        }
        
        .download-button:hover {
          background-color: var(--accent-hover, #3d7dcf);
        }
        
        button:disabled {
          background-color: var(--border-color, #333);
          cursor: not-allowed;
          opacity: 0.7;
        }
        
        .processing {
          opacity: 0.7;
          pointer-events: none;
        }
        
        @media (max-width: 768px) {
          .audio-panel {
            flex-direction: column;
          }
          
          audio {
            width: 100%;
          }
          
          .button-group {
            width: 100%;
            justify-content: space-between;
          }
        }
      </style>
      
      <div class="audio-panel">
        <audio id="audioPlayer" controls></audio>
        
        <div class="button-group">
          <button class="reset-button" id="resetButton">Reset to Original</button>
          <button class="download-button" id="downloadButton">Download Processed</button>
        </div>
      </div>
    `,this.audio=this.root.getElementById("audioPlayer"),this.resetButton=this.root.getElementById("resetButton"),this.downloadButton=this.root.getElementById("downloadButton")}setupEventListeners(){this.audio.addEventListener("play",async e=>{!this.isProcessed&&!this.isProcessing&&(this.audio.pause(),await this.processAudio(),this.isProcessed&&this.audio.play())}),this.resetButton.addEventListener("click",()=>{this.resetAudio()}),this.downloadButton.addEventListener("click",()=>{this.dispatchEvent(new Event("downloadAudio",{bubbles:!0,composed:!0}))})}async processAudio(){if(!this.isProcessing){this.isProcessing=!0,this.audio.classList.add("processing");try{const e=new CustomEvent("processAudio",{bubbles:!0,composed:!0,detail:{callback:this.onProcessingComplete.bind(this)}});this.dispatchEvent(e)}catch(e){console.error("Processing failed:",e),this.onProcessingComplete(!1)}}}onProcessingComplete(e=!0){this.isProcessing=!1,this.isProcessed=e,this.audio.classList.remove("processing")}resetAudio(){this.isProcessed=!1,this.dispatchEvent(new Event("resetAudio",{bubbles:!0,composed:!0}))}updateAudio(e){const t=new Blob([e],{type:"audio/wav"}),o=URL.createObjectURL(t);this.audio.src&&this.audio.src.startsWith("blob:")&&URL.revokeObjectURL(this.audio.src),this.audio.src=o,this.onProcessingComplete(!0)}setEnabled(e){this.resetButton.disabled=!e,this.downloadButton.disabled=!e}resetProcessed(){this.isProcessed=!1}setOriginalAudio(e){this.isProcessed=!1,this.updateAudio(e)}}customElements.define("audio-player",M);function I(i,e,t,o,s){if(i.fillStyle="#2a2a2a",i.fillRect(0,0,e,t),!o||o.length===0){S(i,e,t,"No spectrum data available");return}k(i,e,t),E(i,t),A(i,e,t,s);const r=z(o,3),a=i.createLinearGradient(0,t,0,0);a.addColorStop(0,"rgba(37, 134, 255, 0.7)"),a.addColorStop(.5,"rgba(92, 195, 255, 0.8)"),a.addColorStop(1,"rgba(255, 255, 255, 0.9)"),i.beginPath(),i.moveTo(0,t);const n=e/r.length;for(let l=0;l<r.length;l++){const c=l*n,d=t-r[l]*t;i.lineTo(c,d)}i.lineTo(e,t),i.closePath(),i.fillStyle=a,i.fill(),i.beginPath(),i.moveTo(0,t);for(let l=0;l<r.length;l++){const c=l*n,d=t-r[l]*t;i.lineTo(c,d)}i.strokeStyle="rgba(255, 255, 255, 0.8)",i.lineWidth=2,i.stroke(),O(i,e,t,r,s)}function y(i,e,t){i.fillStyle="#2a2a2a",i.fillRect(0,0,e,t),k(i,e,t),E(i,t),A(i,e,t,{sample_rate:44100}),S(i,e,t,"Load an audio file to see the spectrum")}function S(i,e,t,o){i.fillStyle="#aaaaaa",i.font="16px Arial",i.textAlign="center",i.fillText(o,e/2,t/2)}function k(i,e,t){i.strokeStyle="rgba(100, 100, 100, 0.2)",i.lineWidth=1;for(let s=0;s<10;s++){const r=s*(t/10);i.beginPath(),i.moveTo(0,r),i.lineTo(e,r),i.stroke()}const o=[20,50,100,200,500,1e3,2e3,5e3,1e4,2e4];for(const s of o)if(s>=20&&s<=2e4){const r=C(s,e);i.beginPath(),i.moveTo(r,0),i.lineTo(r,t),i.strokeStyle=s%1e3===0?"rgba(150, 150, 150, 0.3)":"rgba(100, 100, 100, 0.2)",i.stroke()}}function E(i,e){i.fillStyle="#aaaaaa",i.font="12px Arial",i.textAlign="left";const t=[0,.25,.5,.75,1];for(const o of t){const s=e-o*e,r=Math.round(o*100)+"%";i.fillRect(0,s,4,1),i.fillText(r,7,s+4)}}function A(i,e,t,o){if(!o.sample_rate)return;i.fillStyle="#aaaaaa",i.font="12px Arial",i.textAlign="center";const s=[20,50,100,200,500,1e3,2e3,5e3,1e4,2e4];for(const r of s)if(r>=20&&r<=2e4){const a=C(r,e);let n;r>=1e3?n=`${r/1e3}kHz`:n=`${r}Hz`,[20,100,1e3,1e4,2e4].includes(r)&&i.fillText(n,a,t-5)}}function O(i,e,t,o,s){if(!s.sample_rate)return;const r=_(o,.5,Math.floor(o.length/50));if(r.length===0)return;const a=s.sample_rate/2;i.fillStyle="rgba(255, 220, 100, 0.9)",i.font="10px Arial",i.textAlign="center";for(const n of r){const l=n*(e/o.length),c=t-o[n]*t;i.beginPath(),i.arc(l,c,3,0,Math.PI*2),i.fill();const d=n/o.length,p=20,h=a,f=Math.log10(p),m=Math.log10(h),L=f+d*(m-f),b=Math.pow(10,L);let v;b>=1e3?v=`${(b/1e3).toFixed(1)}kHz`:v=`${Math.round(b)}Hz`,i.fillText(v,l,c-10)}}function C(i,e){const s=Math.log10(20),a=Math.log10(2e4)-s;return(Math.log10(i)-s)/a*e}function z(i,e){if(i.length<=e)return i;const t=new Float32Array(i.length);for(let o=0;o<e-1;o++)t[o]=i[o];for(let o=e-1;o<i.length;o++){let s=0;for(let r=0;r<e;r++)s+=i[o-r];t[o]=s/e}return t}function _(i,e,t){const o=[],s=e;for(let r=1;r<i.length-1&&!(i[r]>i[r-1]&&i[r]>i[r+1]&&i[r]>=s&&o.every(n=>Math.abs(r-n)>=t)&&(o.push(r),o.length>=5));r++);return o}class R extends HTMLElement{shadowRoot;canvas;ctx;audioService=null;constructor(){super(),this.shadowRoot=this.attachShadow({mode:"open"})}connectedCallback(){this.render(),this.setupCanvas(),y(this.ctx,this.canvas.width,this.canvas.height)}render(){this.shadowRoot.innerHTML=`
      <style>
        :host {
          display: block;
          margin: 25px 0;
        }
        
        canvas {
          width: 100%;
          height: 300px;
          background-color: var(--canvas-bg, #2a2a2a);
          border-radius: 8px;
          box-shadow: inset 0 0 10px rgba(0, 0, 0, 0.2);
        }
      </style>
      
      <canvas id="spectrumCanvas" width="800" height="300"></canvas>
    `}setupCanvas(){this.canvas=this.shadowRoot.getElementById("spectrumCanvas"),this.ctx=this.canvas.getContext("2d"),new ResizeObserver(t=>{for(const o of t){const{width:s}=o.contentRect;this.canvas.width=s,this.updateSpectrum(this.audioService)}}).observe(this.canvas)}updateSpectrum(e){if(this.audioService=e,!e){y(this.ctx,this.canvas.width,this.canvas.height);return}try{const t=e.getInfo(),o=this.canvas.width,s=e.getSpectrumData(0,o,!0);I(this.ctx,this.canvas.width,this.canvas.height,s,t)}catch(t){console.error("Error drawing spectrum:",t),y(this.ctx,this.canvas.width,this.canvas.height)}}}customElements.define("spectrum-visualizer",R);class u extends HTMLElement{root;audioService=null;controls=new Map;constructor(){super(),this.root=this.attachShadow({mode:"open"})}connectedCallback(){this.render(),this.setupEventListeners()}setAudioService(e){this.audioService=e}render(){const e=this.getConfig();this.root.innerHTML=`
      <style>
        ${this.getStyles()}
      </style>
      
      <div class="operation-content">
        <p class="description">${e.description}</p>
        
        <div class="controls-container">
          ${e.controls.map(t=>this.renderControl(t)).join("")}
        </div>
        
        <button class="apply-button" id="applyButton">Apply ${e.name}</button>
      </div>
    `,e.controls.forEach(t=>{const o=this.root.getElementById(t.id);o&&this.controls.set(t.id,o)})}renderControl(e){switch(e.type){case"slider":return this.renderSlider(e);case"select":return this.renderSelect(e);case"checkbox":return this.renderCheckbox(e);case"number":return this.renderNumberInput(e);case"text":return this.renderTextInput(e);default:return""}}renderSlider(e){const t=e.defaultValue??e.min??0,o=this.formatValue(t,e.unit);return`
      <div class="slider-container">
        <div class="slider-label">
          <span>${e.label}:</span>
          <span class="value-display" id="${e.id}Value">${o}</span>
        </div>
        ${e.help?`<div class="help-text">${e.help}</div>`:""}
        <input 
          type="range" 
          id="${e.id}" 
          min="${e.min??0}" 
          max="${e.max??100}" 
          step="${e.step??1}" 
          value="${t}"
          data-unit="${e.unit||""}"
        >
      </div>
    `}renderSelect(e){return`
      <div class="form-group">
        <label for="${e.id}">${e.label}:</label>
        ${e.help?`<div class="help-text">${e.help}</div>`:""}
        <select id="${e.id}">
          ${e.options?.map(t=>`
            <option value="${t.value}" ${t.value===e.defaultValue?"selected":""}>
              ${t.label}
            </option>
          `).join("")}
        </select>
      </div>
    `}renderCheckbox(e){return`
      <div class="form-group checkbox-group">
        <label>
          <input 
            type="checkbox" 
            id="${e.id}" 
            ${e.defaultValue?"checked":""}
          >
          ${e.label}
        </label>
        ${e.help?`<div class="help-text">${e.help}</div>`:""}
      </div>
    `}renderNumberInput(e){return`
      <div class="form-group">
        <label for="${e.id}">${e.label}:</label>
        ${e.help?`<div class="help-text">${e.help}</div>`:""}
        <input 
          type="number" 
          id="${e.id}" 
          min="${e.min??""}" 
          max="${e.max??""}" 
          step="${e.step??1}" 
          value="${e.defaultValue??""}"
          class="number-input"
        >
      </div>
    `}renderTextInput(e){return`
      <div class="form-group">
        <label for="${e.id}">${e.label}:</label>
        ${e.help?`<div class="help-text">${e.help}</div>`:""}
        <input 
          type="text" 
          id="${e.id}" 
          value="${e.defaultValue??""}"
          class="text-input"
        >
      </div>
    `}setupEventListeners(){const e=this.getConfig();e.controls.forEach(o=>{if(o.type==="slider"){const s=this.controls.get(o.id),r=this.root.getElementById(`${o.id}Value`);s&&r&&s.addEventListener("input",()=>{const a=parseFloat(s.value);r.textContent=this.formatValue(a,o.unit)})}}),this.root.getElementById("applyButton")?.addEventListener("click",()=>{if(!this.audioService){console.error("Audio service not set");return}const o={};e.controls.forEach(s=>{const r=this.controls.get(s.id);if(r)switch(s.type){case"slider":case"number":o[s.id]=parseFloat(r.value);break;case"checkbox":o[s.id]=r.checked;break;case"select":case"text":o[s.id]=r.value;break}});try{this.onApply(o),this.dispatchEvent(new CustomEvent("operationApplied",{detail:{operation:e.name,values:o},bubbles:!0,composed:!0}))}catch(s){console.error(`Error applying ${e.name}:`,s)}})}formatValue(e,t){return t==="Hz"&&e>=1e3?`${(e/1e3).toFixed(1)} kHz`:t==="%"?`${e}%`:t?`${e} ${t}`:e.toString()}getStyles(){return`
      :host {
        display: block;
      }
      
      .operation-content {
        padding: 20px;
        background-color: rgba(255, 255, 255, 0.03);
        border-radius: 8px;
      }
      
      .description {
        font-size: 14px;
        color: var(--text-secondary, #aaa);
        margin-bottom: 20px;
        line-height: 1.4;
      }
      
      .controls-container {
        display: flex;
        flex-direction: column;
        gap: 15px;
      }
      
      /* Form groups */
      .form-group {
        display: flex;
        flex-direction: column;
        gap: 5px;
      }
      
      .checkbox-group {
        flex-direction: row;
        align-items: center;
      }
      
      .checkbox-group label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
      }
      
      label {
        font-size: 14px;
        color: var(--text-secondary, #aaa);
      }
      
      .help-text {
        font-size: 12px;
        color: var(--text-secondary, #888);
        font-style: italic;
      }
      
      /* Slider styles */
      .slider-container {
        margin: 15px 0;
      }
      
      .slider-label {
        display: flex;
        justify-content: space-between;
        margin-bottom: 8px;
      }
      
      .value-display {
        color: var(--accent-color, #4f9eff);
        font-weight: 500;
      }
      
      input[type="range"] {
        width: 100%;
        height: 6px;
        -webkit-appearance: none;
        background-color: var(--slider-track, #444);
        border-radius: 3px;
        outline: none;
      }
      
      input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        width: 16px;
        height: 16px;
        background-color: var(--slider-thumb, #6fb5ff);
        border-radius: 50%;
        cursor: pointer;
        transition: background-color 0.2s;
      }
      
      input[type="range"]::-moz-range-thumb {
        width: 16px;
        height: 16px;
        background-color: var(--slider-thumb, #6fb5ff);
        border: none;
        border-radius: 50%;
        cursor: pointer;
      }
      
      /* Select styles */
      select {
        padding: 8px 12px;
        background-color: rgba(0, 0, 0, 0.2);
        color: var(--text-primary, #e0e0e0);
        border: 1px solid var(--border-color, #333);
        border-radius: 4px;
        font-size: 14px;
        cursor: pointer;
      }
      
      /* Input styles */
      input[type="text"],
      input[type="number"] {
        padding: 8px 12px;
        background-color: rgba(0, 0, 0, 0.2);
        color: var(--text-primary, #e0e0e0);
        border: 1px solid var(--border-color, #333);
        border-radius: 4px;
        font-size: 14px;
      }
      
      input[type="checkbox"] {
        width: 18px;
        height: 18px;
        cursor: pointer;
      }
      
      /* Button styles */
      .apply-button {
        margin-top: 20px;
        padding: 12px 24px;
        background-color: var(--accent-color, #4f9eff);
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        font-weight: 500;
        transition: all 0.2s;
      }
      
      .apply-button:hover {
        background-color: var(--accent-hover, #3d7dcf);
        transform: translateY(-1px);
      }
      
      .apply-button:active {
        transform: translateY(0);
      }
      
      .apply-button:disabled {
        background-color: var(--border-color, #333);
        cursor: not-allowed;
        opacity: 0.7;
      }
    `}}class H extends u{getConfig(){return{name:"Amplitude Power",description:"Raises the amplitude of each frequency bin to the specified power.",controls:[{type:"slider",id:"power",label:"Power Exponent",min:.01,max:5,step:.01,defaultValue:1}]}}onApply(e){this.audioService.applyPower(e.power)}}customElements.define("amplitude-power-operation",H);class V extends u{filterType="lowpass";getConfig(){return{name:"Frequency Filter",description:"Apply lowpass, highpass or  bandpass filter.",controls:[{type:"select",id:"filterType",label:"Filter Type",defaultValue:"lowpass",options:[{value:"lowpass",label:"Lowpass"},{value:"highpass",label:"Highpass"},{value:"bandpass",label:"Bandpass"}]},{type:"slider",id:"cutoffLow",label:"Cutoff Frequency",min:20,max:2e4,step:1,defaultValue:1e3,unit:"Hz"},{type:"slider",id:"cutoffHigh",label:"High Cutoff",min:20,max:2e4,step:1,defaultValue:5e3,unit:"Hz"}]}}connectedCallback(){super.connectedCallback(),this.setupFilterTypeHandling()}setupFilterTypeHandling(){const e=this.controls.get("filterType"),t=this.root.querySelector('[id="cutoffHigh"]')?.closest(".slider-container"),o=this.root.querySelector('[for="cutoffLow"]');e&&t&&(t.style.display="none",e.addEventListener("change",()=>{this.filterType=e.value,this.filterType==="bandpass"?(t.style.display="block",o&&(o.textContent="Low Cutoff:")):(t.style.display="none",o&&(o.textContent="Cutoff Frequency:"))}))}onApply(e){switch(e.filterType){case"lowpass":this.audioService.applyLowpass(e.cutoffLow);break;case"highpass":this.audioService.applyHighpass(e.cutoffLow);break;case"bandpass":this.audioService.applyBandpass(e.cutoffLow,e.cutoffHigh);break}}}customElements.define("frequency-filters-operation",V);class U extends u{getConfig(){return{name:"Phase Multiply",description:"Multiply all phases with the value you specify. A value of -1 will reverse the sound.",controls:[{type:"slider",id:"factor",label:"Phase Factor",min:-3,max:3,step:.01,defaultValue:1}]}}onApply(e){this.audioService.applyPhaseMultiply(e.factor)}}customElements.define("phase-multiply-operation",U);class D extends u{getConfig(){return{name:"Bin Swap",description:"Randomly swaps frequency bins.",controls:[{type:"slider",id:"blockSize",label:"Block Size",min:.01,max:100,step:.01,defaultValue:1,unit:"%",help:"Maximum distance between swapped bins (% of full spectrum)"},{type:"slider",id:"repeat",label:"Number of Swaps",min:.001,max:10,step:.001,defaultValue:.1,unit:"%",help:"Number of swaps to perform (% of number of bins)"}]}}onApply(e){this.audioService.swapBins(e.blockSize,e.repeat)}}customElements.define("bin-swap-operation",D);class W extends u{getConfig(){return{name:"Channel Swap",description:"Randomly swaps frequency bins between different channels. Only works with multi-channel audio (stereo or more).",controls:[{type:"slider",id:"repeat",label:"Number of Swaps",min:.001,max:10,step:.001,defaultValue:.1,unit:"%",help:"Percentage of bins to swap between channels"}]}}onApply(e){if(this.audioService.getInfo().channels<2)throw new Error("Channel swapping requires at least 2 channels (stereo audio)");this.audioService.swapChannels(e.repeat)}}customElements.define("channel-swap-operation",W);class N extends u{getConfig(){return{name:"Spectrum Shift",description:"Shifts the entire frequency spectrum up or down by the specified amount in Hz.",controls:[{type:"slider",id:"shiftAmount",label:"Shift Amount",min:-5e3,max:5e3,step:1,defaultValue:0,unit:"Hz",help:"Positive values shift up, negative values shift down"}]}}onApply(e){this.audioService.applySpectrumShift(e.shiftAmount)}}customElements.define("spectrum-shift-operation",N);class j extends u{getConfig(){return{name:"Stretch",description:"Applies non-linear stretching to the frequency spectrum.",controls:[{type:"slider",id:"exponent",label:"Stretch Exponent",min:.2,max:5,step:.1,defaultValue:1.3,help:"Higher values compress high frequencies more"}]}}onApply(e){this.audioService.applyStretch(e.exponent)}}customElements.define("frequency-stretch-operation",j);class K extends u{getConfig(){return{name:"Wobble",description:"This transform will alternately stretch and contract the frequency axis using a sinusoidal transfer function for the frequencies._The Frequency parameter controls the number of periods of the transfer function from 0 Hz to the Nyquist frequency, while Amplitude controls its amplitude (1 is the entire frequency axis).",controls:[{type:"slider",id:"frequency",label:"Frequency",min:1,max:5e3,step:1,defaultValue:10,help:"Number of periods of the transfer function"},{type:"slider",id:"amplitude",label:"Amplitude",min:.001,max:.1,step:.001,defaultValue:.01,help:"Amount"}]}}onApply(e){this.audioService.applyWobble(e.frequency,e.amplitude)}}customElements.define("wobble-operation",K);class G extends u{getConfig(){return{name:"Threshold",description:"Removes all partials below a given amplitude threshold.",controls:[{type:"slider",id:"level",label:"Threshold Level",min:.01,max:5,step:.01,defaultValue:1},{type:"checkbox",id:"removeAbove",label:"Remove components above threshold (instead of below)",defaultValue:!1,help:"Check to remove frequencies above the threshold (peak limiter), uncheck to remove below (noise gate)"}]}}onApply(e){this.audioService.applyThreshold(e.level,e.removeAbove)}}customElements.define("threshold-operation",G);class Y extends u{getConfig(){return{name:"Amplitude Derivative",description:"Replaces each frequency bin's amplitude with the difference between it and the previous bin.",controls:[{type:"slider",id:"multiplier",label:"Derivative Multiplier",min:.1,max:10,step:.1,defaultValue:1,help:"Scaling factor for the derivative values"}]}}onApply(e){this.audioService.applyAmplitudeDerivative(e.multiplier)}}customElements.define("amplitude-derivative-operation",Y);class Q extends u{getConfig(){return{name:"Keep Peaks",description:"Keeps only the local maxima in the frequency spectrum, zeroing out all other bins.",controls:[]}}onApply(e){this.audioService.keepPeaks()}}customElements.define("keep-peaks-operation",Q);class X extends u{getConfig(){return{name:"Spectrum Split",description:"Split the audio into multiple files, each containing only specific frequency bins. This allows separation of frequency components across multiple files.",controls:[{type:"text",id:"baseName",label:"Base Filename",defaultValue:"split",help:'Base name for output files (e.g., "split" produces "split_0.wav", "split_1.wav", etc.)'},{type:"number",id:"numParts",label:"Number of Parts",min:2,max:16,step:1,defaultValue:2,help:"How many files to split the spectrum into"},{type:"number",id:"groupSize",label:"Group Size",min:0,max:32,step:1,defaultValue:1,help:"Number of consecutive frequency bins to group together, 0 = num bins/num parts"},{type:"checkbox",id:"distributeLog",label:"Use log distribution",defaultValue:!1,help:"Check to distribute the frequencies for each part using a log distribution"}]}}async onApply(e){const{baseName:t,numParts:o,groupSize:s,distributeLog:r}=e;this.setProcessing(!0,`Splitting into ${o} parts...`);try{for(let a=0;a<o;a++){this.updateProgress(`Processing part ${a+1}/${o}...`),this.audioService.prepareSplitPart(a,o,s,r),await this.audioService.processAudio();const n=`${t}_${a}.wav`;this.audioService.downloadProcessedAudio(n),this.audioService.resetSplit(),await new Promise(l=>setTimeout(l,100))}this.updateProgress(`Split completed! Downloaded ${o} files.`)}finally{this.setProcessing(!1)}}setProcessing(e,t){const o=this.root.querySelector(".apply-button");o&&(o.disabled=e,t?o.textContent=t:o.textContent=`Apply ${this.getConfig().name}`)}updateProgress(e){const t=this.root.querySelector(".apply-button");t&&(t.textContent=e)}}customElements.define("spectrum-split-operation",X);class J extends u{noteFrequencies=new Map([["C",261.63],["C#",277.18],["D",293.66],["D#",311.13],["E",329.63],["F",349.23],["F#",369.99],["G",392],["G#",415.3],["A",440],["A#",466.16],["B",493.88]]);getConfig(){return{name:"Chord Filter",description:"Filter audio to keep only the frequencies of a chord and their harmonics.",controls:[...Array.from({length:5},(e,t)=>[{type:"number",id:`freq${t+1}`,label:`Frequency ${t+1}`,min:0,max:2e4,step:.1,defaultValue:t===0?440:0,unit:"Hz"},{type:"slider",id:`amp${t+1}`,label:`Amplitude ${t+1}`,min:0,max:1,step:.01,defaultValue:t===0?1:0}]).flat(),{type:"slider",id:"width",label:"Width",min:0,max:50,step:1,defaultValue:25,unit:"cents",help:"Width around each frequency to keep"},{type:"slider",id:"harmonics",label:"Harmonic Strength",min:0,max:1,step:.01,defaultValue:.5,help:"0: Only fundamental, 1: Full harmonic series"}]}}connectedCallback(){super.connectedCallback(),this.addPianoKeyboard()}addPianoKeyboard(){const e=this.root.querySelector(".controls-container");if(e){const t=`
        <div class="piano-section">
          <h4>Quick Note Selection</h4>
          <div class="octave-selector">
            ${Array.from({length:8},(o,s)=>`<button class="octave-btn ${s===4?"active":""}" data-octave="${s}">${s}</button>`).join("")}
          </div>
          <div class="piano-keyboard">
            ${this.renderPianoKeys()}
          </div>
        </div>
      `;e.insertAdjacentHTML("afterend",t),this.setupPianoEventListeners()}}renderPianoKeys(){const e=["C","D","E","F","G","A","B"],t=["C#","D#",null,"F#","G#","A#"];return`
      <div class="white-keys">
        ${e.map(o=>`<button class="key white-key" data-note="${o}">${o}</button>`).join("")}
      </div>
      <div class="black-keys">
        ${t.map(o=>o?`<button class="key black-key" data-note="${o}">${o}</button>`:'<div class="black-key-space"></div>').join("")}
      </div>
    `}setupPianoEventListeners(){let e=4;this.root.querySelectorAll(".octave-btn").forEach(t=>{t.addEventListener("click",o=>{this.root.querySelectorAll(".octave-btn").forEach(s=>s.classList.remove("active")),o.target.classList.add("active"),e=parseInt(o.target.dataset.octave)})}),this.root.querySelectorAll(".key").forEach(t=>{t.addEventListener("click",o=>{const s=o.target.dataset.note,r=this.noteFrequencies.get(s)*Math.pow(2,e-4);this.addFrequencyToNextSlot(r)})})}addFrequencyToNextSlot(e){for(let t=1;t<=5;t++){const o=this.controls.get(`freq${t}`);if(o&&(parseFloat(o.value)===0||o.value==="")){o.value=e.toFixed(2);const s=this.controls.get(`amp${t}`);s&&(s.value="1");break}}}onApply(e){const t=[e.freq1,e.freq2,e.freq3,e.freq4,e.freq5],o=[e.amp1,e.amp2,e.amp3,e.amp4,e.amp5];this.audioService.applyChordFilter(t,o,e.width,e.harmonics)}getStyles(){return super.getStyles()+`
      .piano-section {
        margin-top: 20px;
        padding: 15px;
        background-color: rgba(0, 0, 0, 0.15);
        border-radius: 8px;
      }
      
      .piano-section h4 {
        margin: 0 0 10px 0;
        color: var(--text-primary);
      }
      
      .octave-selector {
        display: flex;
        gap: 5px;
        margin-bottom: 10px;
      }
      
      .octave-btn {
        width: 30px;
        height: 30px;
        border: 1px solid var(--border-color);
        background: rgba(255, 255, 255, 0.1);
        color: var(--text-primary);
        cursor: pointer;
        border-radius: 4px;
      }
      
      .octave-btn.active {
        background: var(--accent-color);
        color: white;
      }
      
      .piano-keyboard {
        position: relative;
        height: 80px;
      }
      
      .white-keys, .black-keys {
        display: flex;
        position: absolute;
        width: 100%;
      }
      
      .white-keys {
        height: 100%;
      }
      
      .black-keys {
        height: 60%;
        padding: 0 2%;
      }
      
      .key {
        border: 1px solid #333;
        cursor: pointer;
        display: flex;
        align-items: flex-end;
        justify-content: center;
        padding-bottom: 5px;
        font-size: 12px;
      }
      
      .white-key {
        flex: 1;
        background: white;
        color: black;
        margin: 0 1px;
      }
      
      .black-key {
        width: 8%;
        background: #333;
        color: white;
        margin: 0 1%;
      }
      
      .black-key-space {
        width: 8%;
        margin: 0 1%;
      }
      
      .key:active {
        background: var(--accent-color);
        color: white;
      }
    `}}customElements.define("chord-filter-operation",J);class Z extends u{convolveFile=null;getConfig(){return{name:"Convolution",description:"Convolve or correlate your audio with another audio file.",controls:[{type:"select",id:"mode",label:"Operation Mode",defaultValue:"convolve",options:[{value:"convolve",label:"Convolution"},{value:"correlate",label:"Correlation"}],help:"Convolution time-reverses the IR before multiplying spectra"},{type:"slider",id:"wetMix",label:"Wet/Dry Mix",min:0,max:1,step:.01,defaultValue:1,help:"0 = 100% dry (original), 1 = 100% wet (processed)"}]}}connectedCallback(){super.connectedCallback(),this.addFileInput()}addFileInput(){const e=this.root.querySelector(".controls-container");if(e){e.insertAdjacentHTML("afterbegin",`
        <div class="form-group file-input-group">
          <label for="convolveFile">Select IR/Second File:</label>
          <div class="file-input-wrapper">
            <input type="file" id="convolveFile" accept="audio/*" style="display: none;">
            <button class="file-select-button" id="selectFileButton">Choose File</button>
            <span id="fileInfo" class="file-info">No file selected</span>
          </div>
        </div>
      `);const o=this.root.getElementById("convolveFile"),s=this.root.getElementById("selectFileButton"),r=this.root.getElementById("fileInfo");s?.addEventListener("click",()=>o.click()),o.addEventListener("change",a=>{const n=a.target;if(n.files&&n.files.length>0){this.convolveFile=n.files[0];const l=(this.convolveFile.size/1024).toFixed(1);r.textContent=`${this.convolveFile.name} (${l} KB)`}})}}async onApply(e){if(!this.convolveFile)throw new Error("Please select a file for convolution");const t=await this.convolveFile.arrayBuffer(),o=new Uint8Array(t),s=e.mode==="correlate";await this.audioService.convolveWithFile(o,s,e.wetMix)}getStyles(){return super.getStyles()+`
      .file-input-group {
        margin-bottom: 15px;
        padding-bottom: 15px;
        border-bottom: 1px solid var(--border-color, #333);
      }
      
      .file-input-wrapper {
        display: flex;
        align-items: center;
        gap: 10px;
        margin-top: 8px;
      }
      
      .file-select-button {
        padding: 8px 16px;
        background-color: #444;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        transition: background-color 0.2s;
      }
      
      .file-select-button:hover {
        background-color: #555;
      }
      
      .file-info {
        font-size: 13px;
        color: var(--text-secondary, #aaa);
      }
    `}}customElements.define("convolution-operation",Z);class ee extends HTMLElement{shadowRoot;audioService=null;activeTab="power";tabs=[{id:"power",label:"Amplitude Power",component:"amplitude-power-operation"},{id:"phase-mult",label:"Phase Multiplication",component:"phase-multiply-operation"},{id:"bin-swap",label:"Frequency Bin Swap",component:"bin-swap-operation"},{id:"channel-swap",label:"Channel Bin Swap",component:"channel-swap-operation"},{id:"shift",label:"Spectrum Shift",component:"spectrum-shift-operation"},{id:"stretch",label:"Stretch",component:"frequency-stretch-operation"},{id:"wobble",label:"Wobble",component:"wobble-operation"},{id:"threshold",label:"Threshold",component:"threshold-operation"},{id:"derivate",label:"Derivative Amplitude",component:"amplitude-derivative-operation"},{id:"keep-peaks",label:"Keep Peaks",component:"keep-peaks-operation"},{id:"split",label:"Frequency Spectrum Split",component:"spectrum-split-operation"},{id:"filters",label:"Filters",component:"frequency-filters-operation"},{id:"chord-filter",label:"Chord Filter",component:"chord-filter-operation"},{id:"convolution",label:"Convolution",component:"convolution-operation"}];constructor(){super(),this.shadowRoot=this.attachShadow({mode:"open"})}connectedCallback(){this.render(),this.setupEventListeners()}setAudioService(e){this.audioService=e,this.tabs.forEach(t=>{this.shadowRoot.querySelector(t.component)?.setAudioService(e)})}render(){this.shadowRoot.innerHTML=`
      <style>
        :host {
          display: block;
          margin: 30px 0;
        }
        
        .tabs-container {
          background-color: var(--card-bg, #1e1e1e);
          border-radius: 8px;
          overflow: hidden;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
        }
        
        .tabs-header {
          display: flex;
          flex-wrap: wrap;
          background-color: rgba(0, 0, 0, 0.2);
          border-bottom: 1px solid var(--border-color, #333);
        }
        
        .tab-button {
          flex: 1 0 auto;
          min-width: 120px;
          padding: 12px 20px;
          background: none;
          border: none;
          color: var(--text-secondary, #aaa);
          font-size: 15px;
          cursor: pointer;
          transition: all 0.3s ease;
          position: relative;
        }
        
        .tab-button:hover {
          background-color: rgba(76, 175, 80, 0.1);
          color: var(--text-primary, #e0e0e0);
        }
        
        .tab-button.active {
          color: white;
          font-weight: 500;
          background-color: var(--success-color, #4CAF50);
        }
        
        .tab-button.active::after {
          content: '';
          position: absolute;
          bottom: 0;
          left: 0;
          width: 100%;
          height: 3px;
          background-color: var(--success-color, #4CAF50);
        }
        
        .tabs-content {
          padding: 20px;
        }
        
        .tab-content {
          display: none;
        }
        
        .tab-content.active {
          display: block;
        }
      </style>
      
      <div class="tabs-container">
        <div class="tabs-header">
          ${this.tabs.map(e=>`
            <button class="tab-button ${e.id===this.activeTab?"active":""}" 
                    data-tab="${e.id}">
              ${e.label}
            </button>
          `).join("")}
        </div>
        
        <div class="tabs-content">
          ${this.tabs.map(e=>`
            <div class="tab-content ${e.id===this.activeTab?"active":""}" 
                 id="${e.id}-tab">
              <${e.component}></${e.component}>
            </div>
          `).join("")}
        </div>
      </div>
    `}setupEventListeners(){this.shadowRoot.querySelectorAll(".tab-button").forEach(t=>{t.addEventListener("click",o=>{const r=o.target.dataset.tab;this.switchTab(r)})})}switchTab(e){this.activeTab=e,this.shadowRoot.querySelectorAll(".tab-button").forEach(t=>{t.classList.toggle("active",t.getAttribute("data-tab")===e)}),this.shadowRoot.querySelectorAll(".tab-content").forEach(t=>{t.classList.toggle("active",t.id===`${e}-tab`)})}}customElements.define("operation-tabs",ee);class te extends HTMLElement{shadowRoot;statusElement;timeoutId=null;constructor(){super(),this.shadowRoot=this.attachShadow({mode:"open"})}connectedCallback(){this.render()}render(){this.shadowRoot.innerHTML=`
      <style>
        :host {
          display: block;
          margin: 20px 0;
        }
        
        .status {
          background-color: rgba(255, 255, 255, 0.05);
          border-left: 4px solid var(--accent-color, #4f9eff);
          padding: 12px 18px;
          font-size: 14px;
          color: var(--text-primary, #e0e0e0);
          border-radius: 0 4px 4px 0;
          transition: all 0.3s ease;
          min-height: 20px;
        }
        
        .status.error {
          background-color: var(--error-bg, rgba(244, 67, 54, 0.1));
          border-left-color: var(--error-color, #f44336);
          color: var(--error-color, #f44336);
        }
        
        .status.success {
          background-color: var(--success-bg, rgba(76, 175, 80, 0.1));
          border-left-color: var(--success-color, #4CAF50);
          color: var(--success-color, #4CAF50);
        }
        
        .status.hidden {
          opacity: 0;
          transform: translateY(-10px);
        }
      </style>
      
      <div class="status" id="statusElement">
        Ready to load audio file. Choose or drop a WAV file to begin.
      </div>
    `,this.statusElement=this.shadowRoot.getElementById("statusElement")}setStatus(e,t="info",o=!1){this.timeoutId&&(clearTimeout(this.timeoutId),this.timeoutId=null),this.statusElement.textContent=e,this.statusElement.className="status",t==="error"?this.statusElement.classList.add("error"):t==="success"&&this.statusElement.classList.add("success"),this.statusElement.classList.remove("hidden"),(o||t==="success")&&(this.timeoutId=window.setTimeout(()=>{this.statusElement.classList.add("hidden")},5e3))}}customElements.define("status-bar",te);async function w(){const i=document.getElementById("app");if(!i){console.error("App element not found!");return}try{i.innerHTML='<div class="loading">Loading WASM module...</div>',console.log("Initializing WASM..."),await x.initialize(),console.log("WASM initialized successfully"),i.innerHTML="";const e=document.createElement("audio-processor-app");i.appendChild(e),console.log("App component mounted")}catch(e){console.error("Failed to initialize application:",e),i.innerHTML=`
      <div class="error">
        <h1>Failed to load application</h1>
        <p>Error: ${e}</p>
        <p>Check the console for more details.</p>
      </div>
    `}}document.readyState==="loading"?document.addEventListener("DOMContentLoaded",w):w();
