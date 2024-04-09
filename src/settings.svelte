<script lang="ts">
  let fft_plot_decay = 0.8;
  let fft_plot_size = 256;

  // this will need to rewrite redis.conf...right?
  let server_update_interval = 30;
  let server_update_num_changes = 5;
</script>

<form>
  <div class="wrapper">
    <div class="item">
      <span class="group-label">plot scale</span>
      <span><input type="radio" name="plot_scale" checked /> linear</span>
      <span><input type="radio" name="plot_scale" /> mel</span>
      <span><input type="radio" name="plot_scale" /> log</span>
      <span><input type="radio" name="plot_scale" /> bark</span>
    </div>

    <div class="item">
      <span class="group-label">fft plot decay</span>
      <input type="range" name="fft_plot_decay" value={fft_plot_decay} />
      <span style="width:100%;">{fft_plot_decay}</span>
    </div>

    <div class="item">
      <span class="group-label">fft plot size</span>
      <span><input type="radio" name="fft_plot_size" /> 64</span>
      <span><input type="radio" name="fft_plot_size" /> 128</span>
      <span><input type="radio" name="fft_plot_size" checked /> 256</span>
      <span><input type="radio" name="fft_plot_size" /> 512</span>
    </div>

    <div class="item">
      <span class="group-label">theme</span>
      <span><input type="radio" name="theme" /> rgb</span>
      <span><input type="radio" name="theme" /> cym</span>
      <span><input type="radio" name="theme" /> pog</span>
      <span><input type="radio" name="theme" /> custom</span>
    </div>

    <div class="item">
      <span class="group-label">server</span>
      <input
        type="range"
        name="server_update_interval"
        value={server_update_interval}
      />
      <span style="width:100%;">{server_update_interval}</span>
      <input
        type="range"
        name="server_update_num_changes"
        value={server_update_num_changes}
      />
      <span style="width:100%;">{server_update_num_changes}</span>
    </div>

    <div class="wide-item">
      <span class="group-label">colors</span>
      <span><input type="color" />rotary ticks</span>
      <span><input type="color" />rotary hover</span>
      <span><input type="color" />slider border</span>
      <span><input type="color" />slider fader</span>
      <span><input type="color" />slider hover</span>
      <span><input type="color" />slider mouse down</span>
      <span><input type="color" />plot main</span>
      <span><input type="color" />plot single filter </span>
      <span><input type="color" />plot sum filters</span>
    </div>
  </div>
</form>

<style>
  .wrapper {
    position: absolute;
    display: grid;
    flex-direction: row;
    top: 45px;
    left: 25px;
    border: 1px solid black;
    width: 95%;
    background: rgba(130, 130, 130, 0.8);
    color: black;
    gap: 1em;
    grid-template-rows: auto auto;
    grid-template-columns: auto auto auto;
    justify-items: center;
  }
  .item {
    display: flex;
    flex-direction: column;
    align-items: start;
    text-align: center;
  }
  .wide-item {
    display: grid;
    text-align: center;
    grid-template-rows: auto auto auto;
    grid-template-columns: auto auto;
    align-items: start;
    justify-items: start;
    gap: 0.3em;
  }
  .group-label {
    text-decoration: underline;
    grid-column: 1 / span 2;
    align-self: center;
    justify-self: center;
  }
</style>
