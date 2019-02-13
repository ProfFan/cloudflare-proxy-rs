<template>
  <div class="dns-record">
    <h1>This is a record</h1>
    <h3>{{ id }}</h3>
  </div>
</template>

<script>
  export default {
    name: "Record.vue",
    props: {
      id: String
    }
  }
</script>

<style scoped>

</style>