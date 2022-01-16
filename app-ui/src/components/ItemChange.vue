<template>
  <form>
    <div class="field">
      <label class="label">Item Name</label>
      <div class="control">
        <input
          class="input"
          type="text"
          placeholder="Name"
          v-model="itemName"
          required
        />
      </div>
      <p class="help">Required</p>
    </div>
    <div class="field">
      <label class="label">Item SKU</label>
      <div class="control">
        <input class="input" type="text" placeholder="SKU" v-model="sku" />
      </div>
    </div>
    <div class="field">
      <label class="label">Item Supplier</label>
      <div class="control">
        <input
          class="input"
          type="text"
          placeholder="Supplier"
          v-model="supplier"
        />
      </div>
    </div>
    <div class="field">
      <label class="label">Item Description</label>
      <div class="control">
        <textarea
          class="textarea"
          type="textarea"
          placeholder="Description"
          v-model="description"
        ></textarea>
      </div>
    </div>
    <div class="field is-grouped">
      <div class="control">
        <input
          class="button is-primary"
          type="submit"
          value="Submit"
          v-on:click="submitItem"
        />
      </div>
      <div class="control">
        <input
          v-if="this.$route.params.id"
          class="button"
          type="button"
          value="Cancel"
          v-on:click="$emit('submit')"
        />
      </div>
    </div>
  </form>
</template>
<script>
import { useRoute } from 'vue-router';
import { useMutation } from '@urql/vue';

export default {
  setup() {
    const route = useRoute();
    const [actionString, variableString, variableAssignment] = route.params.id
      ? ['update', '$id: ItemId!, ', 'id: $id, ']
      : ['create', '', ''];
    const changeItem = useMutation(`
      mutation (${variableString}$item: InsertableItem!) {
        ${actionString}Item(${variableAssignment}item: $item) {
          id
        }
      }
    `);

    return {
      changeItem(name, sku, supplier, description) {
        let variables = {
          item: Object.assign(
            {
              name: name,
            },
            sku ? { sku: sku } : null,
            supplier ? { supplier: supplier } : null,
            description ? { description: description } : null
          ),
        };
        Object.assign(
          variables,
          route.params.id ? { id: parseInt(route.params.id) } : null
        );
        changeItem.executeMutation(variables).then((result) => {
          if (result.error) {
            alert(JSON.stringify(result.error.graphQLErrors[0]));
          } else {
            this.$emit('submit');
            alert('Changed!');
          }
        });
      },
    };
  },
  props: [
    'initialItemName',
    'initialSku',
    'initialSupplier',
    'initialDescription',
  ],
  data: function () {
    return {
      itemName: this.initialItemName,
      sku: this.initialSku,
      supplier: this.initialSupplier,
      description: this.initialDescription,
    };
  },
  methods: {
    submitItem: function (e) {
      e.preventDefault();
      this.changeItem(this.itemName, this.sku, this.supplier, this.description);
    },
  },
};
</script>