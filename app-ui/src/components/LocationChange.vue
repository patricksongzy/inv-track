<template>
  <form>
    <div class="field">
      <label class="label">Location Name</label>
      <div class="control">
        <input
          class="input"
          type="text"
          placeholder="Name"
          v-model="locationName"
          required
        />
      </div>
      <p class="help">Required</p>
    </div>
    <div class="field">
      <label class="label">Location Address</label>
      <div class="control">
        <input
          class="input"
          type="text"
          placeholder="Address"
          v-model="address"
        />
      </div>
    </div>
    <div class="field is-grouped">
      <div class="control">
        <input
          class="button is-primary"
          type="submit"
          value="Submit"
          v-on:click="submitLocation"
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
    const changeLocation = useMutation(`
      mutation (${variableString}$location: InsertableLocation!) {
        ${actionString}Location(${variableAssignment}location: $location) {
          id
        }
      }
    `);

    return {
      changeLocation(name, address) {
        let variables = {
          location: Object.assign(
            {
              name: name,
            },
            address ? { address: address } : null
          ),
        };
        Object.assign(
          variables,
          route.params.id ? { id: parseInt(route.params.id) } : null
        );
        changeLocation.executeMutation(variables).then((result) => {
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
  props: ['initialLocationName', 'initialAddress'],
  data: function () {
    return {
      locationName: this.initialLocationName,
      address: this.initialAddress,
    };
  },
  methods: {
    submitLocation: function (e) {
      e.preventDefault();
      this.changeLocation(this.locationName, this.address);
    },
  },
};
</script>